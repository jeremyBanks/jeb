(async () => {
  const buildTimestampPath = "/last-build-timestamp.json";
  let firstTimestamp: number;
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
