"use strict";
(async () => {
  const buildTimestampPath = "/last-build-timestamp.json";
  let firstTimestamp;
  let previousTimestamp = undefined;
  try {
    firstTimestamp = await (
      await fetch(buildTimestampPath, { cache: "no-store" })
    ).json();
  } catch (error) {
    console.info(
      "Auto-reload not enabled because build timestamp file not found.",
      error,
    );
    return;
  }
  while (true) {
    let currentTimestamp;
    try {
      currentTimestamp = await (await fetch(buildTimestampPath)).json();
    } catch (error) {
      if (previousTimestamp || true) {
        console.debug(
          "Failed to load build timestamp file; a new build may be in progress.",
          error,
        );
      }
      // Once we know a build is in progress, poll rapidly for its completion.
      await new Promise(resolve => setTimeout(resolve, 250));
      previousTimestamp = undefined;
      continue;
    }
    if (currentTimestamp !== firstTimestamp) {
      console.info("New build timestamp detected, auto-reloading.");
      document.location.reload();
      await new Promise(resolve => setTimeout(resolve, 4000));
      console.error("Did we fail to reload?");
    }
    // Build usually takes longer than this so we don't need to poll more often.
    await new Promise(resolve => setTimeout(resolve, 8000));
    previousTimestamp = currentTimestamp;
  }
})();
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiYXV0b3JlbG9hZC5qcyIsInNvdXJjZVJvb3QiOiIuLyIsInNvdXJjZXMiOlsiZm9yZWdyb3VuZC9hdXRvcmVsb2FkLnRzIl0sIm5hbWVzIjpbXSwibWFwcGluZ3MiOiI7QUFBQSxDQUFDLEtBQUssSUFBSSxFQUFFO0lBQ1YsTUFBTSxrQkFBa0IsR0FBRyw0QkFBNEIsQ0FBQztJQUN4RCxJQUFJLGNBQXNCLENBQUM7SUFDM0IsSUFBSSxpQkFBaUIsR0FBRyxTQUFTLENBQUM7SUFFbEMsSUFBSTtRQUNGLGNBQWMsR0FBRyxNQUFNLENBQ3JCLE1BQU0sS0FBSyxDQUFDLGtCQUFrQixFQUFFLEVBQUUsS0FBSyxFQUFFLFVBQVUsRUFBRSxDQUFDLENBQ3ZELENBQUMsSUFBSSxFQUFFLENBQUM7S0FDVjtJQUFDLE9BQU8sS0FBSyxFQUFFO1FBQ2QsT0FBTyxDQUFDLElBQUksQ0FDVixpRUFBaUUsRUFDakUsS0FBSyxDQUNOLENBQUM7UUFDRixPQUFPO0tBQ1I7SUFFRCxPQUFPLElBQUksRUFBRTtRQUNYLElBQUksZ0JBQWdCLENBQUM7UUFDckIsSUFBSTtZQUNGLGdCQUFnQixHQUFHLE1BQU0sQ0FBQyxNQUFNLEtBQUssQ0FBQyxrQkFBa0IsQ0FBQyxDQUFDLENBQUMsSUFBSSxFQUFFLENBQUM7U0FDbkU7UUFBQyxPQUFPLEtBQUssRUFBRTtZQUNkLElBQUksaUJBQWlCLElBQUksSUFBSSxFQUFFO2dCQUM3QixPQUFPLENBQUMsS0FBSyxDQUNYLHNFQUFzRSxFQUN0RSxLQUFLLENBQ04sQ0FBQzthQUNIO1lBQ0Qsd0VBQXdFO1lBQ3hFLE1BQU0sSUFBSSxPQUFPLENBQUMsT0FBTyxDQUFDLEVBQUUsQ0FBQyxVQUFVLENBQUMsT0FBTyxFQUFFLEdBQUcsQ0FBQyxDQUFDLENBQUM7WUFDdkQsaUJBQWlCLEdBQUcsU0FBUyxDQUFDO1lBQzlCLFNBQVM7U0FDVjtRQUVELElBQUksZ0JBQWdCLEtBQUssY0FBYyxFQUFFO1lBQ3ZDLE9BQU8sQ0FBQyxJQUFJLENBQUMsK0NBQStDLENBQUMsQ0FBQztZQUM5RCxRQUFRLENBQUMsUUFBUSxDQUFDLE1BQU0sRUFBRSxDQUFDO1lBQzNCLE1BQU0sSUFBSSxPQUFPLENBQUMsT0FBTyxDQUFDLEVBQUUsQ0FBQyxVQUFVLENBQUMsT0FBTyxFQUFFLElBQUksQ0FBQyxDQUFDLENBQUM7WUFDeEQsT0FBTyxDQUFDLEtBQUssQ0FBQyx3QkFBd0IsQ0FBQyxDQUFDO1NBQ3pDO1FBRUQsNEVBQTRFO1FBQzVFLE1BQU0sSUFBSSxPQUFPLENBQUMsT0FBTyxDQUFDLEVBQUUsQ0FBQyxVQUFVLENBQUMsT0FBTyxFQUFFLElBQUksQ0FBQyxDQUFDLENBQUM7UUFDeEQsaUJBQWlCLEdBQUcsZ0JBQWdCLENBQUM7S0FDdEM7QUFDSCxDQUFDLENBQUMsRUFBRSxDQUFDIiwic291cmNlc0NvbnRlbnQiOlsiKGFzeW5jICgpID0+IHtcbiAgY29uc3QgYnVpbGRUaW1lc3RhbXBQYXRoID0gXCIvbGFzdC1idWlsZC10aW1lc3RhbXAuanNvblwiO1xuICBsZXQgZmlyc3RUaW1lc3RhbXA6IG51bWJlcjtcbiAgbGV0IHByZXZpb3VzVGltZXN0YW1wID0gdW5kZWZpbmVkO1xuXG4gIHRyeSB7XG4gICAgZmlyc3RUaW1lc3RhbXAgPSBhd2FpdCAoXG4gICAgICBhd2FpdCBmZXRjaChidWlsZFRpbWVzdGFtcFBhdGgsIHsgY2FjaGU6IFwibm8tc3RvcmVcIiB9KVxuICAgICkuanNvbigpO1xuICB9IGNhdGNoIChlcnJvcikge1xuICAgIGNvbnNvbGUuaW5mbyhcbiAgICAgIFwiQXV0by1yZWxvYWQgbm90IGVuYWJsZWQgYmVjYXVzZSBidWlsZCB0aW1lc3RhbXAgZmlsZSBub3QgZm91bmQuXCIsXG4gICAgICBlcnJvcixcbiAgICApO1xuICAgIHJldHVybjtcbiAgfVxuXG4gIHdoaWxlICh0cnVlKSB7XG4gICAgbGV0IGN1cnJlbnRUaW1lc3RhbXA7XG4gICAgdHJ5IHtcbiAgICAgIGN1cnJlbnRUaW1lc3RhbXAgPSBhd2FpdCAoYXdhaXQgZmV0Y2goYnVpbGRUaW1lc3RhbXBQYXRoKSkuanNvbigpO1xuICAgIH0gY2F0Y2ggKGVycm9yKSB7XG4gICAgICBpZiAocHJldmlvdXNUaW1lc3RhbXAgfHwgdHJ1ZSkge1xuICAgICAgICBjb25zb2xlLmRlYnVnKFxuICAgICAgICAgIFwiRmFpbGVkIHRvIGxvYWQgYnVpbGQgdGltZXN0YW1wIGZpbGU7IGEgbmV3IGJ1aWxkIG1heSBiZSBpbiBwcm9ncmVzcy5cIixcbiAgICAgICAgICBlcnJvcixcbiAgICAgICAgKTtcbiAgICAgIH1cbiAgICAgIC8vIE9uY2Ugd2Uga25vdyBhIGJ1aWxkIGlzIGluIHByb2dyZXNzLCBwb2xsIHJhcGlkbHkgZm9yIGl0cyBjb21wbGV0aW9uLlxuICAgICAgYXdhaXQgbmV3IFByb21pc2UocmVzb2x2ZSA9PiBzZXRUaW1lb3V0KHJlc29sdmUsIDI1MCkpO1xuICAgICAgcHJldmlvdXNUaW1lc3RhbXAgPSB1bmRlZmluZWQ7XG4gICAgICBjb250aW51ZTtcbiAgICB9XG5cbiAgICBpZiAoY3VycmVudFRpbWVzdGFtcCAhPT0gZmlyc3RUaW1lc3RhbXApIHtcbiAgICAgIGNvbnNvbGUuaW5mbyhcIk5ldyBidWlsZCB0aW1lc3RhbXAgZGV0ZWN0ZWQsIGF1dG8tcmVsb2FkaW5nLlwiKTtcbiAgICAgIGRvY3VtZW50LmxvY2F0aW9uLnJlbG9hZCgpO1xuICAgICAgYXdhaXQgbmV3IFByb21pc2UocmVzb2x2ZSA9PiBzZXRUaW1lb3V0KHJlc29sdmUsIDQwMDApKTtcbiAgICAgIGNvbnNvbGUuZXJyb3IoXCJEaWQgd2UgZmFpbCB0byByZWxvYWQ/XCIpO1xuICAgIH1cblxuICAgIC8vIEJ1aWxkIHVzdWFsbHkgdGFrZXMgbG9uZ2VyIHRoYW4gdGhpcyBzbyB3ZSBkb24ndCBuZWVkIHRvIHBvbGwgbW9yZSBvZnRlbi5cbiAgICBhd2FpdCBuZXcgUHJvbWlzZShyZXNvbHZlID0+IHNldFRpbWVvdXQocmVzb2x2ZSwgODAwMCkpO1xuICAgIHByZXZpb3VzVGltZXN0YW1wID0gY3VycmVudFRpbWVzdGFtcDtcbiAgfVxufSkoKTtcbiJdfQ==
