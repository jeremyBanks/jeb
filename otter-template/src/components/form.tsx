import { css } from "emotion";
import React, { ReactNode, useState } from "react";

export let Form = ({
  children,
  onSubmit
}: {
  children: ReactNode;
  onSubmit: (data: FormData) => void;
}) => {
  let [disabled, setDisabled] = useState(false);

  return (
    <form
      onSubmit={(event) => {
        event.preventDefault();

        let nativeEvent: any = event.nativeEvent;
        let data = new FormData(nativeEvent.target);
        let submitter: HTMLInputElement | undefined =
          nativeEvent.submitter ||
          nativeEvent.target.querySelector("fieldset :enabled");
        let submitterName = submitter?.name || "value";
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
