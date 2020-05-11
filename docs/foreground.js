import "./foreground/autoreload.js";
import "./foreground/jsx.js";
import * as views from "./foreground/views.js";
import { fragment } from "./foreground/jsx.js";
import { spider } from "./foreground/spider.js";
export const main = async () => {
  document.head.appendChild(
    fragment(
      JSX.createElement("title", null, "stadia.observer"),
      JSX.createElement("link", {
        rel: "icon",
        href: "/illufinch-violetsky-edited@2x.png",
      }),
    ),
  );
  if (browser?.runtime?.id) {
    document.body.appendChild(
      JSX.createElement(
        "button",
        { onclick: spider, style: { cursor: "pointer" } },
        "\uD83D\uDD77\uFE0F Spider Stadia Store",
      ),
    );
  }
  // TODO: associate prototypes, these are plain Objects.
  const skus = Object.values(await (await fetch("/skus.json")).json());
  document.body.appendChild(JSX.createElement(views.Home, { skus: skus }));
};
main().catch(error => console.error("Unexpected error in main.", error));
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiZm9yZWdyb3VuZC5qcyIsInNvdXJjZVJvb3QiOiIuLyIsInNvdXJjZXMiOlsiZm9yZWdyb3VuZC50c3giXSwibmFtZXMiOltdLCJtYXBwaW5ncyI6IkFBQUEsT0FBTyw0QkFBNEIsQ0FBQztBQUNwQyxPQUFPLHFCQUFxQixDQUFDO0FBRzdCLE9BQU8sS0FBSyxLQUFLLE1BQU0sdUJBQXVCLENBQUM7QUFFL0MsT0FBTyxFQUFFLFFBQVEsRUFBRSxNQUFNLHFCQUFxQixDQUFDO0FBQy9DLE9BQU8sRUFBRSxNQUFNLEVBQUUsTUFBTSx3QkFBd0IsQ0FBQztBQUVoRCxNQUFNLENBQUMsTUFBTSxJQUFJLEdBQUcsS0FBSyxJQUFJLEVBQUU7SUFDN0IsUUFBUSxDQUFDLElBQUksQ0FBQyxXQUFXLENBQ3ZCLFFBQVEsQ0FDTixtREFBOEIsRUFDOUIsNEJBQU0sR0FBRyxFQUFDLE1BQU0sRUFBQyxJQUFJLEVBQUMsb0NBQW9DLEdBQUcsQ0FDOUQsQ0FDRixDQUFDO0lBRUYsSUFBSSxPQUFPLEVBQUUsT0FBTyxFQUFFLEVBQUUsRUFBRTtRQUN4QixRQUFRLENBQUMsSUFBSSxDQUFDLFdBQVcsQ0FDdkIsOEJBQVEsT0FBTyxFQUFFLE1BQU0sRUFBRSxLQUFLLEVBQUUsRUFBRSxNQUFNLEVBQUUsU0FBUyxFQUFFLDZDQUU1QyxDQUNWLENBQUM7S0FDSDtJQUVELHVEQUF1RDtJQUN2RCxNQUFNLElBQUksR0FBc0IsTUFBTSxDQUFDLE1BQU0sQ0FDM0MsTUFBTSxDQUFDLE1BQU0sS0FBSyxDQUFDLFlBQVksQ0FBQyxDQUFDLENBQUMsSUFBSSxFQUFFLENBQ3pDLENBQUM7SUFFRixRQUFRLENBQUMsSUFBSSxDQUFDLFdBQVcsQ0FBQyxrQkFBQyxLQUFLLENBQUMsSUFBSSxJQUFDLElBQUksRUFBRSxJQUFJLEdBQUksQ0FBQyxDQUFDO0FBQ3hELENBQUMsQ0FBQztBQUVGLElBQUksRUFBRSxDQUFDLEtBQUssQ0FBQyxLQUFLLENBQUMsRUFBRSxDQUFDLE9BQU8sQ0FBQyxLQUFLLENBQUMsMkJBQTJCLEVBQUUsS0FBSyxDQUFDLENBQUMsQ0FBQyIsInNvdXJjZXNDb250ZW50IjpbImltcG9ydCBcIi4vZm9yZWdyb3VuZC9hdXRvcmVsb2FkLmpzXCI7XG5pbXBvcnQgXCIuL2ZvcmVncm91bmQvanN4LmpzXCI7XG5cbmltcG9ydCAqIGFzIG1vZGVscyBmcm9tIFwiLi9mb3JlZ3JvdW5kL2RhdGEvbW9kZWxzLmpzXCI7XG5pbXBvcnQgKiBhcyB2aWV3cyBmcm9tIFwiLi9mb3JlZ3JvdW5kL3ZpZXdzLmpzXCI7XG5cbmltcG9ydCB7IGZyYWdtZW50IH0gZnJvbSBcIi4vZm9yZWdyb3VuZC9qc3guanNcIjtcbmltcG9ydCB7IHNwaWRlciB9IGZyb20gXCIuL2ZvcmVncm91bmQvc3BpZGVyLmpzXCI7XG5cbmV4cG9ydCBjb25zdCBtYWluID0gYXN5bmMgKCkgPT4ge1xuICBkb2N1bWVudC5oZWFkLmFwcGVuZENoaWxkKFxuICAgIGZyYWdtZW50KFxuICAgICAgPHRpdGxlPnN0YWRpYS5vYnNlcnZlcjwvdGl0bGU+LFxuICAgICAgPGxpbmsgcmVsPVwiaWNvblwiIGhyZWY9XCIvaWxsdWZpbmNoLXZpb2xldHNreS1lZGl0ZWRAMngucG5nXCIgLz4sXG4gICAgKSxcbiAgKTtcblxuICBpZiAoYnJvd3Nlcj8ucnVudGltZT8uaWQpIHtcbiAgICBkb2N1bWVudC5ib2R5LmFwcGVuZENoaWxkKFxuICAgICAgPGJ1dHRvbiBvbmNsaWNrPXtzcGlkZXJ9IHN0eWxlPXt7IGN1cnNvcjogXCJwb2ludGVyXCIgfX0+XG4gICAgICAgIPCflbfvuI8gU3BpZGVyIFN0YWRpYSBTdG9yZVxuICAgICAgPC9idXR0b24+LFxuICAgICk7XG4gIH1cblxuICAvLyBUT0RPOiBhc3NvY2lhdGUgcHJvdG90eXBlcywgdGhlc2UgYXJlIHBsYWluIE9iamVjdHMuXG4gIGNvbnN0IHNrdXM6IEFycmF5PG1vZGVscy5Ta3U+ID0gT2JqZWN0LnZhbHVlcyhcbiAgICBhd2FpdCAoYXdhaXQgZmV0Y2goXCIvc2t1cy5qc29uXCIpKS5qc29uKCksXG4gICk7XG5cbiAgZG9jdW1lbnQuYm9keS5hcHBlbmRDaGlsZCg8dmlld3MuSG9tZSBza3VzPXtza3VzfSAvPik7XG59O1xuXG5tYWluKCkuY2F0Y2goZXJyb3IgPT4gY29uc29sZS5lcnJvcihcIlVuZXhwZWN0ZWQgZXJyb3IgaW4gbWFpbi5cIiwgZXJyb3IpKTtcbiJdfQ==
