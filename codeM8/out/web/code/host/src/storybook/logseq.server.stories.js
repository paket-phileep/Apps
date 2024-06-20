// import { action } from "@storybook/addon-actions";
import Logseq from './logseq';

export default {
  title: 'Server/Logseq',
  component: Logseq,
  parameters: {
    // layout: 'centered',
  },
  argTypes: {
    label: { control: 'text' },
    backgroundColor: { control: 'color' },
    width: { control: 'text' },
    height: { control: 'text' },
    borderColor: { control: 'color' },
    borderSize: { control: 'number' },
    pattern: { control: 'number' },
    tailwind: {
      control: {
        type: 'select',
        options: [
          'isometric',
          'bg-blue-500',
          'bg-green-500',
          // Add more tailwind classes as needed
        ],
      },
    },
  },
  args: {
    // onClick: action("onClick"),
    label: 'Logseq',
    width: '100',
    height: '100',
    dotSize: 8,
    dotColor: 'blue',
  },
};

export const Static = (args) => <Logseq {...args} static />;

Static.args = {
  label: 'Logseq',
  width: '100',
  dotSize: 8,
  dotColor: 'blue',
};

export const Dynamic = (args) => <Logseq {...args} />;

Dynamic.args = {
  label: 'Logseq',
  width: '100',
  dotSize: 8,
  dotColor: 'blue',
};
