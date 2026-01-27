import React, { ReactElement, ReactNode } from "react";

import { Form } from "../components/form";
import { print } from "../utilities/print";

/**
 * Displays a form, and disables it and returns the entered FormData when the
 * user submits it.
 */
export let formInput = async (children: ReactNode): Promise<FormData> =>
  new Promise((resolve) => {
    print(<Form onSubmit={resolve}>{children}</Form>);
  });

/**
 * Returns a Record with the values from each form control by name.
 *
 * If there are duplicate names they will be overwritten, use `formInput`
 * instead if you need to capture those.
 */
export let inputs = async (
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

/**
 * Returns the value specified with a single form control.
 *
 * Defaults to a text input, optionally with a default string to be used if the
 * input is empty.
 */
export let input = async (child?: string | ReactElement): Promise<string> => {
  if (typeof child === "string") {
    child = <input autoFocus placeholder={child} />;
  } else if (typeof child === "undefined") {
    child = <input autoFocus />;
  }
  return (await inputs(child)).value || "";
};
