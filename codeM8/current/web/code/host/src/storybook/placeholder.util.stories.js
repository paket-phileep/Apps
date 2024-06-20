// import { action } from "@storybook/addon-actions";
import Placeholder from "./placeholder";

export default {
  title: "Util/Placeholder",
  component: Placeholder,
  parameters: {
    layout: "centered",
  },
  argTypes: {
    label: { control: "text" },
    backgroundColor: { control: "color" },
    width: { control: "text" },
    height: { control: "text" },
    borderColor: { control: "color" },
    borderSize: { control: "number" },
    pattern: { control: "number" },
    tailwind: {
      control: {
        type: "select",
        options: [
          "isometric",
          "bg-blue-500",
          "bg-green-500",
          // Add more tailwind classes as needed
        ],
      },
    },
  },
  args: {
    // onClick: action("onClick"),
    label: "Placeholder",
    width: "100",
    height: "100",
    dotSize: 8,
    dotColor: "blue",
  },
};

export const Static = (args) => <Placeholder {...args} static />;

Static.args = {
  label: "Placeholder",
  width: "100",
  dotSize: 8,
  dotColor: "blue",
};

export const Dynamic = (args) => <Placeholder {...args} />;

Dynamic.args = {
  label: "Placeholder",
  width: "100",
  dotSize: 8,
  dotColor: "blue",
};
