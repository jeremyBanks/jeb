import { css } from "emotion";
import React, { ReactNode, useState } from "react";

export const Form: React.FC<{
  children: ReactNode;
  onSubmit(data: FormData): void;
}> = ({ children, onSubmit }) => {
  const [disabled, setDisabled] = useState(false);

  return (
    <form
      onSubmit={(event) => {
        event.preventDefault();

        const nativeEvent: any = event.nativeEvent;
        const data = new FormData(nativeEvent.target);
        const submitter: HTMLInputElement | undefined =
          nativeEvent.submitter ||
          nativeEvent.target.querySelector("fieldset :enabled");
        const submitterName = submitter?.name || "value";
        if (submitter && !data.has(submitterName)) {
          let value =
            submitter.value ||
            submitter.placeholder ||
            submitter.textContent ||
            "";
          data.append(submitterName, value);
        }
        setDisabled(true);
        if (submitter) {
          submitter.blur();
        }
        onSubmit(data);
      }}
    >
      <fieldset
        disabled={disabled}
        className={css({
          display: "contents"
        })}
      >
        {children}
      </fieldset>
    </form>
  );
};
