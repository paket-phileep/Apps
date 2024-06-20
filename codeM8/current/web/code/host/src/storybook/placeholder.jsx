import React from "react";
//  fix this later 

const Placeholder = ({ height = 400 }) => {
  const [size, setSize] = React.useState(`${height}px`); // Initial size

  const divStyle = {
    "--s": size,
    "--c1": "#fcf6e5",
    "--c2": "#e5dfcf",
    "--c3": "#697a82",
    "--_g": "0 120deg, #0000 0",
    background: `
      conic-gradient(from 0deg at calc(500% / 6) calc(100% / 3), var(--c3) var(--_g)),
      conic-gradient(from -120deg at calc(100% / 6) calc(100% / 3), var(--c2) var(--_g)),
      conic-gradient(from 120deg at calc(100% / 3) calc(500% / 6), var(--c1) var(--_g)),
      conic-gradient(from 120deg at calc(200% / 3) calc(500% / 6), var(--c1) var(--_g)),
      conic-gradient(from -180deg at calc(100% / 3) 50%, var(--c2) 60deg, var(--c1) var(--_g)),
      conic-gradient(from 60deg at calc(200% / 3) 50%, var(--c1) 60deg, var(--c3) var(--_g)),
      conic-gradient(from -60deg at 50% calc(100% / 3), var(--c1) 120deg, var(--c2) 0 240deg, var(--c3) 0)`,
    backgroundSize: `calc(${size} * 1.732) ${size}`,
  };

  // Example function to change size
  const handleSizeChange = () => {
    setSize("150px"); // Change size to 150px (or any other value)
  };

  return (
    <div style={divStyle}>
      <button onClick={handleSizeChange}>Change Size</button>
      <p>Your content here</p>
    </div>
  );
};

export default Placeholder;
