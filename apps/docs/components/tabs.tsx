"use client";

import { Tab as FumaTab, Tabs as FumaTabs, type TabProps, type TabsProps } from "fumadocs-ui/components/tabs";
import React from "react";

// derive tab labels when callers omit items
const Tabs = ({ children, items, ...rest }: TabsProps) => {
  const titles =
    items ??
    React.Children.toArray(children)
      .filter(React.isValidElement)
      .map(
        (child) =>
          (child.props as { title?: string; value?: string }).title ?? (child.props as { value?: string }).value
      )
      .filter((title): title is string => Boolean(title));

  return (
    <FumaTabs items={titles} {...rest}>
      {children}
    </FumaTabs>
  );
};

const Tab = ({ title, value, ...props }: TabProps & { title?: string }) => (
  <FumaTab value={value ?? title} {...props} />
);

export { Tabs, Tab };
