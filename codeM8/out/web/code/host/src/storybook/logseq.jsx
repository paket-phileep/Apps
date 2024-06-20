import React from 'react';
import IframeComponent from '../../util/iframe-component';
//  fix this later

import app from '../../constants/apps.json';

const Placeholder = ({ height = 400 }) => {
  const [size, setSize] = React.useState(`${height}px`); // Initial size

  return <IframeComponent url={app['logseq']} />;
};

export default Placeholder;
