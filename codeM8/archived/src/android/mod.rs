// Copyright 2020-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::{PageLoadEvent, WebContext, WebViewAttributes, RGBA};
use crate::{RequestAsyncResponder, Result};
use base64::{engine::general_purpose, Engine};
use crossbeam_channel::*;
use html5ever::{interface::QualName, namespace_url, ns, tendril::TendrilSink, LocalName};
use http::{
  header::{HeaderValue, CONTENT_SECURITY_POLICY, CONTENT_TYPE},
  Request, Response as HttpResponse,
};
use jni::{
  errors::Result as JniResult,
  objects::{GlobalRef, JClass, JObject},
  JNIEnv,
};
use kuchiki::NodeRef;
use ndk::looper::{FdEvent, ForeignLooper};
use once_cell::sync::OnceCell;
use raw_window_handle::HasWindowHandle;
use sha2::{Digest, Sha256};
use std::{
  borrow::Cow,
  collections::HashMap,
  sync::{atomic::AtomicI32, mpsc::channel, Mutex},
};

pub(crate) mod binding;
mod main_pipe;
use main_pipe::{CreateWebViewAttributes, MainPipe, WebViewMessage, MAIN_PIPE};

pub struct Context<'a, 'b> {
  pub env: &'a mut JNIEnv<'b>,
  pub activity: &'a JObject<'b>,
  pub webview: &'a JObject<'b>,
}

macro_rules! define_static_handlers {
  ($($var:ident = $type_name:ident { $($fields:ident:$types:ty),+ $(,)? });+ $(;)?) => {
    $(pub static $var: once_cell::sync::OnceCell<$type_name> = once_cell::sync::OnceCell::new();
    pub struct $type_name {
      $($fields: $types,)*
    }
    impl $type_name {
      pub fn new($($fields: $types,)*) -> Self {
        Self {
          $($fields,)*
        }
      }
    }
    unsafe impl Send for $type_name {}
    unsafe impl Sync for $type_name {})*
  };
}

define_static_handlers! {
  IPC =  UnsafeIpc { handler: Box<dyn Fn(Request<String>)> };
  REQUEST_HANDLER = UnsafeRequestHandler { handler:  Box<dyn Fn(Request<Vec<u8>>, bool) -> Option<HttpResponse<Cow<'static, [u8]>>>> };
  TITLE_CHANGE_HANDLER = UnsafeTitleHandler { handler: Box<dyn Fn(String)> };
  URL_LOADING_OVERRIDE = UnsafeUrlLoadingOverride { handler: Box<dyn Fn(String) -> bool> };
  ON_LOAD_HANDLER = UnsafeOnPageLoadHandler { handler: Box<dyn Fn(PageLoadEvent, String)> };
}

pub static WITH_ASSET_LOADER: OnceCell<bool> = OnceCell::new();
pub static ASSET_LOADER_DOMAIN: OnceCell<String> = OnceCell::new();

pub(crate) static PACKAGE: OnceCell<String> = OnceCell::new();

type EvalCallback = Box<dyn Fn(String) + Send + 'static>;

pub static EVAL_ID_GENERATOR: OnceCell<AtomicI32> = OnceCell::new();
pub static EVAL_CALLBACKS: once_cell::sync::OnceCell<Mutex<HashMap<i32, EvalCallback>>> =
  once_cell::sync::OnceCell::new();

/// Sets up the necessary logic for wry to be able to create the webviews later.
pub unsafe fn android_setup(
  package: &str,
  mut env: JNIEnv,
  looper: &ForeignLooper,
  activity: GlobalRef,
) {
  PACKAGE.get_or_init(move || package.to_string());

  // we must create the WebChromeClient here because it calls `registerForActivityResult`,
  // which gives an `LifecycleOwners must call register before they are STARTED.` error when called outside the onCreate hook
  let rust_webchrome_client_class = find_class(
    &mut env,
    activity.as_obj(),
    format!("{}/RustWebChromeClient", PACKAGE.get().unwrap()),
  )
  .unwrap();
  let webchrome_client = env
    .new_object(
      &rust_webchrome_client_class,
      &format!("(L{}/WryActivity;)V", PACKAGE.get().unwrap()),
      &[activity.as_obj().into()],
    )
    .unwrap();

  let webchrome_client = env.new_global_ref(webchrome_client).unwrap();
  let mut main_pipe = MainPipe {
    env,
    activity,
    webview: None,
    webchrome_client,
  };

  looper
    .add_fd_with_callback(MAIN_PIPE[0], FdEvent::INPUT, move |_| {
      let size = std::mem::size_of::<bool>();
      let mut wake = false;
      if libc::read(MAIN_PIPE[0], &mut wake as *mut _ as *mut _, size) == size as libc::ssize_t {
        main_pipe.recv().is_ok()
      } else {
        false
      }
    })
    .unwrap();
}

pub(crate) struct InnerWebView;

impl InnerWebView {
  pub fn new_as_child(
    _window: &impl HasWindowHandle,
    attributes: WebViewAttributes,
    pl_attrs: super::PlatformSpecificWebViewAttributes,
    _web_context: Option<&mut WebContext>,
  ) -> Result<Self> {
    Self::new(_window, attributes, pl_attrs, _web_context)
  }

  pub fn new(
    _window: &impl HasWindowHandle,
    attributes: WebViewAttributes,
    pl_attrs: super::PlatformSpecificWebViewAttributes,
    _web_context: Option<&mut WebContext>,
  ) -> Result<Self> {
    let WebViewAttributes {
      url,
      html,
      initialization_scripts,
      ipc_handler,
      #[cfg(any(debug_assertions, feature = "devtools"))]
      devtools,
      custom_protocols,
      background_color,
      transparent,
      headers,
      autoplay,
      user_agent,
      ..
    } = attributes;

    let super::PlatformSpecificWebViewAttributes {
      on_webview_created,
      with_asset_loader,
      asset_loader_domain,
      https_scheme,
    } = pl_attrs;

    let scheme = if https_scheme { "https" } else { "http" };

    let url = if let Some(mut url) = url {
      if let Some(pos) = url.find("://") {
        let name = &url[..pos];
        let is_custom_protocol = custom_protocols.iter().any(|(n, _)| n == name);
        if is_custom_protocol {
          url = url.replace(&format!("{name}://"), &format!("{scheme}://{name}."))
        }
      }

      Some(url)
    } else {
      None
    };

    MainPipe::send(WebViewMessage::CreateWebView(CreateWebViewAttributes {
      url,
      html,
      #[cfg(any(debug_assertions, feature = "devtools"))]
      devtools,
      background_color,
      transparent,
      headers,
      on_webview_created,
      autoplay,
      user_agent,
      initialization_scripts: initialization_scripts.clone(),
    }));

    WITH_ASSET_LOADER.get_or_init(move || with_asset_loader);
    if let Some(domain) = asset_loader_domain {
      ASSET_LOADER_DOMAIN.get_or_init(move || domain);
    }

    REQUEST_HANDLER.get_or_init(move || {
      UnsafeRequestHandler::new(Box::new(
        move |mut request, is_document_start_script_enabled| {
          if let Some(custom_protocol) = custom_protocols.iter().find(|(name, _)| {
            request
              .uri()
              .to_string()
              .starts_with(&format!("{scheme}://{}.", name))
          }) {
            *request.uri_mut() = request
              .uri()
              .to_string()
              .replace(
                &format!("{scheme}://{}.", custom_protocol.0),
                &format!("{}://", custom_protocol.0),
              )
              .parse()
              .unwrap();

            let (tx, rx) = channel();
            let initialization_scripts = initialization_scripts.clone();
            let responder: Box<dyn FnOnce(HttpResponse<Cow<'static, [u8]>>)> =
              Box::new(move |mut response| {
                if !is_document_start_script_enabled {
                  #[cfg(feature = "tracing")]
                  tracing::info!("`addDocumentStartJavaScript` is not supported; injecting initialization scripts via custom protocol handler");
                  let should_inject_scripts = response
                    .headers()
                    .get(CONTENT_TYPE)
                    // Content-Type must begin with the media type, but is case-insensitive.
                    // It may also be followed by any number of semicolon-delimited key value pairs.
                    // We don't care about these here.
                    // source: https://httpwg.org/specs/rfc9110.html#rfc.section.8.3.1
                    .and_then(|content_type| content_type.to_str().ok())
                    .map(|content_type_str| {
                      content_type_str.to_lowercase().starts_with("text/html")
                    })
                    .unwrap_or_default();

                  if should_inject_scripts && !initialization_scripts.is_empty() {
                    let mut document = kuchiki::parse_html()
                      .one(String::from_utf8_lossy(response.body()).into_owned());
                    let csp = response.headers_mut().get_mut(CONTENT_SECURITY_POLICY);
                    let mut hashes = Vec::new();
                    with_html_head(&mut document, |head| {
                      // iterate in reverse order since we are prepending each script to the head tag
                      for script in initialization_scripts.iter().rev() {
                        let script_el = NodeRef::new_element(
                          QualName::new(None, ns!(html), "script".into()),
                          None,
                        );
                        script_el.append(NodeRef::new_text(script));
                        head.prepend(script_el);
                        if csp.is_some() {
                          hashes.push(hash_script(script));
                        }
                      }
                    });

                    if let Some(csp) = csp {
                      let csp_string = csp.to_str().unwrap().to_string();
                      let csp_string = if csp_string.contains("script-src") {
                        csp_string
                          .replace("script-src", &format!("script-src {}", hashes.join(" ")))
                      } else {
                        format!("{} script-src {}", csp_string, hashes.join(" "))
                      };
                      *csp = HeaderValue::from_str(&csp_string).unwrap();
                    }

                    *response.body_mut() = document.to_string().into_bytes().into();
                  }
                }

                tx.send(response).unwrap();
              });

            (custom_protocol.1)(request, RequestAsyncResponder { responder });
            return Some(rx.recv().unwrap());
          }
          None
        },
      ))
    });

    if let Some(i) = ipc_handler {
      IPC.get_or_init(move || UnsafeIpc::new(Box::new(i)));
    }

    if let Some(i) = attributes.document_title_changed_handler {
      TITLE_CHANGE_HANDLER.get_or_init(move || UnsafeTitleHandler::new(i));
    }

    if let Some(i) = attributes.navigation_handler {
      URL_LOADING_OVERRIDE.get_or_init(move || UnsafeUrlLoadingOverride::new(i));
    }

    if let Some(h) = attributes.on_page_load_handler {
      ON_LOAD_HANDLER.get_or_init(move || UnsafeOnPageLoadHandler::new(h));
    }

    Ok(Self)
  }

  pub fn print(&self) -> crate::Result<()> {
    Ok(())
  }

  pub fn url(&self) -> crate::Result<String> {
    let (tx, rx) = bounded(1);
    MainPipe::send(WebViewMessage::GetUrl(tx));
    rx.recv().map_err(Into::into)
  }

  pub fn eval(&self, js: &str, callback: Option<impl Fn(String) + Send + 'static>) -> Result<()> {
    MainPipe::send(WebViewMessage::Eval(
      js.into(),
      callback.map(|c| Box::new(c) as Box<dyn Fn(String) + Send + 'static>),
    ));
    Ok(())
  }

  #[cfg(any(debug_assertions, feature = "devtools"))]
  pub fn open_devtools(&self) {}

  #[cfg(any(debug_assertions, feature = "devtools"))]
  pub fn close_devtools(&self) {}

  #[cfg(any(debug_assertions, feature = "devtools"))]
  pub fn is_devtools_open(&self) -> bool {
    false
  }

  pub fn zoom(&self, _scale_factor: f64) -> Result<()> {
    Ok(())
  }

  pub fn set_background_color(&self, background_color: RGBA) -> Result<()> {
    MainPipe::send(WebViewMessage::SetBackgroundColor(background_color));
    Ok(())
  }

  pub fn load_url(&self, url: &str) -> Result<()> {
    MainPipe::send(WebViewMessage::LoadUrl(url.to_string(), None));
    Ok(())
  }

  pub fn load_url_with_headers(&self, url: &str, headers: http::HeaderMap) -> Result<()> {
    MainPipe::send(WebViewMessage::LoadUrl(url.to_string(), Some(headers)));
    Ok(())
  }

  pub fn clear_all_browsing_data(&self) -> Result<()> {
    MainPipe::send(WebViewMessage::ClearAllBrowsingData);
    Ok(())
  }

  pub fn bounds(&self) -> Result<crate::Rect> {
    Ok(crate::Rect::default())
  }

  pub fn set_bounds(&self, _bounds: crate::Rect) -> Result<()> {
    // Unsupported
    Ok(())
  }

  pub fn set_visible(&self, _visible: bool) -> Result<()> {
    // Unsupported
    Ok(())
  }

  pub fn focus(&self) -> Result<()> {
    // Unsupported
    Ok(())
  }
}

#[derive(Clone, Copy)]
pub struct JniHandle;

impl JniHandle {
  /// Execute jni code on the thread of the webview.
  /// Provided function will be provided with the jni evironment, Android activity and WebView
  pub fn exec<F>(&self, func: F)
  where
    F: FnOnce(&mut JNIEnv, &JObject, &JObject) + Send + 'static,
  {
    MainPipe::send(WebViewMessage::Jni(Box::new(func)));
  }
}

pub fn platform_webview_version() -> Result<String> {
  let (tx, rx) = bounded(1);
  MainPipe::send(WebViewMessage::GetWebViewVersion(tx));
  rx.recv().unwrap()
}

fn with_html_head<F: FnOnce(&NodeRef)>(document: &mut NodeRef, f: F) {
  if let Ok(ref node) = document.select_first("head") {
    f(node.as_node())
  } else {
    let node = NodeRef::new_element(
      QualName::new(None, ns!(html), LocalName::from("head")),
      None,
    );
    f(&node);
    document.prepend(node)
  }
}

fn hash_script(script: &str) -> String {
  let mut hasher = Sha256::new();
  hasher.update(script);
  let hash = hasher.finalize();
  format!("'sha256-{}'", general_purpose::STANDARD.encode(hash))
}

/// Finds a class in the project scope.
pub fn find_class<'a>(
  env: &mut JNIEnv<'a>,
  activity: &JObject<'_>,
  name: String,
) -> JniResult<JClass<'a>> {
  let class_name = env.new_string(name.replace('/', "."))?;
  let my_class = env
    .call_method(
      activity,
      "getAppClass",
      "(Ljava/lang/String;)Ljava/lang/Class;",
      &[(&class_name).into()],
    )?
    .l()?;
  Ok(my_class.into())
}

/// Dispatch a closure to run on the Android context.
///
/// The closure takes the JNI env, the Android activity instance and the possibly null webview.
pub fn dispatch<F>(func: F)
where
  F: FnOnce(&mut JNIEnv, &JObject, &JObject) + Send + 'static,
{
  MainPipe::send(WebViewMessage::Jni(Box::new(func)));
}
