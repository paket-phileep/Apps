import React from 'react';

const IframeComponent = ({ url }) => {
  return (
    <iframe
      src={url}
      style={{ width: '100%', height: '100vh', border: 'none' }}
      title="Embedded Site"
    />
  );
};

export default IframeComponent;
