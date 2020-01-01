import React, { ReactElement, ReactNode } from "react";

import { Form } from "../components/form";
import { print } from "../utilities/print";

export const formInput = async (children: ReactNode): Promise<FormData> =>
  new Promise((resolve) => {
    print(<Form onSubmit={resolve}>{children}</Form>);
  });

export const inputs = async (
  children: ReactNode
): Promise<Record<string, string>> =>
  Object.fromEntries(
    [...(await formInput(children)).entries()].flatMap(([key, value]) => {
      if (value instanceof File) {
        return [];
      } else {
        return [[key, value]];
      }
    })
  );

export const input = async (child?: string | ReactElement): Promise<string> => {
  if (typeof child === "string") {
    child = <input autoFocus placeholder={child} />;
  } else if (typeof child === "undefined") {
    child = <input autoFocus />;
  }
  return (await inputs(child)).value || "";
};
