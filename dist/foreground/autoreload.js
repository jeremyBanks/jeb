"use strict";
(async () => {
    const buildTimestampPath = "/last-build-timestamp.json";
    let firstTimestamp;
    let previousTimestamp = undefined;
    try {
        firstTimestamp = await (await fetch(buildTimestampPath, { cache: "no-store" })).json();
    }
    catch (error) {
        console.info("Auto-reload not enabled because build timestamp file not found.", error);
        return;
    }
    while (true) {
        let currentTimestamp;
        try {
            currentTimestamp = await (await fetch(buildTimestampPath)).json();
        }
        catch (error) {
            if (previousTimestamp) {
                console.debug("Failed to load build timestamp file; a new build may be in progress.", error);
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
        await new Promise(resolve => setTimeout(resolve, 4000));
        previousTimestamp = currentTimestamp;
    }
})();
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiYXV0b3JlbG9hZC5qcyIsInNvdXJjZVJvb3QiOiIuLyIsInNvdXJjZXMiOlsiZm9yZWdyb3VuZC9hdXRvcmVsb2FkLnRzIl0sIm5hbWVzIjpbXSwibWFwcGluZ3MiOiI7QUFBQSxDQUFDLEtBQUssSUFBSSxFQUFFO0lBQ1YsTUFBTSxrQkFBa0IsR0FBRyw0QkFBNEIsQ0FBQztJQUN4RCxJQUFJLGNBQXNCLENBQUM7SUFDM0IsSUFBSSxpQkFBaUIsR0FBRyxTQUFTLENBQUM7SUFFbEMsSUFBSTtRQUNGLGNBQWMsR0FBRyxNQUFNLENBQ3JCLE1BQU0sS0FBSyxDQUFDLGtCQUFrQixFQUFFLEVBQUUsS0FBSyxFQUFFLFVBQVUsRUFBRSxDQUFDLENBQ3ZELENBQUMsSUFBSSxFQUFFLENBQUM7S0FDVjtJQUFDLE9BQU8sS0FBSyxFQUFFO1FBQ2QsT0FBTyxDQUFDLElBQUksQ0FDVixpRUFBaUUsRUFDakUsS0FBSyxDQUNOLENBQUM7UUFDRixPQUFPO0tBQ1I7SUFFRCxPQUFPLElBQUksRUFBRTtRQUNYLElBQUksZ0JBQWdCLENBQUM7UUFDckIsSUFBSTtZQUNGLGdCQUFnQixHQUFHLE1BQU0sQ0FBQyxNQUFNLEtBQUssQ0FBQyxrQkFBa0IsQ0FBQyxDQUFDLENBQUMsSUFBSSxFQUFFLENBQUM7U0FDbkU7UUFBQyxPQUFPLEtBQUssRUFBRTtZQUNkLElBQUksaUJBQWlCLEVBQUU7Z0JBQ3JCLE9BQU8sQ0FBQyxLQUFLLENBQ1gsc0VBQXNFLEVBQ3RFLEtBQUssQ0FDTixDQUFDO2FBQ0g7WUFDRCx3RUFBd0U7WUFDeEUsTUFBTSxJQUFJLE9BQU8sQ0FBQyxPQUFPLENBQUMsRUFBRSxDQUFDLFVBQVUsQ0FBQyxPQUFPLEVBQUUsR0FBRyxDQUFDLENBQUMsQ0FBQztZQUN2RCxpQkFBaUIsR0FBRyxTQUFTLENBQUM7WUFDOUIsU0FBUztTQUNWO1FBRUQsSUFBSSxnQkFBZ0IsS0FBSyxjQUFjLEVBQUU7WUFDdkMsT0FBTyxDQUFDLElBQUksQ0FBQywrQ0FBK0MsQ0FBQyxDQUFDO1lBQzlELFFBQVEsQ0FBQyxRQUFRLENBQUMsTUFBTSxFQUFFLENBQUM7WUFDM0IsTUFBTSxJQUFJLE9BQU8sQ0FBQyxPQUFPLENBQUMsRUFBRSxDQUFDLFVBQVUsQ0FBQyxPQUFPLEVBQUUsSUFBSSxDQUFDLENBQUMsQ0FBQztZQUN4RCxPQUFPLENBQUMsS0FBSyxDQUFDLHdCQUF3QixDQUFDLENBQUM7U0FDekM7UUFFRCw0RUFBNEU7UUFDNUUsTUFBTSxJQUFJLE9BQU8sQ0FBQyxPQUFPLENBQUMsRUFBRSxDQUFDLFVBQVUsQ0FBQyxPQUFPLEVBQUUsSUFBSSxDQUFDLENBQUMsQ0FBQztRQUN4RCxpQkFBaUIsR0FBRyxnQkFBZ0IsQ0FBQztLQUN0QztBQUNILENBQUMsQ0FBQyxFQUFFLENBQUMiLCJzb3VyY2VzQ29udGVudCI6WyIoYXN5bmMgKCkgPT4ge1xuICBjb25zdCBidWlsZFRpbWVzdGFtcFBhdGggPSBcIi9sYXN0LWJ1aWxkLXRpbWVzdGFtcC5qc29uXCI7XG4gIGxldCBmaXJzdFRpbWVzdGFtcDogbnVtYmVyO1xuICBsZXQgcHJldmlvdXNUaW1lc3RhbXAgPSB1bmRlZmluZWQ7XG5cbiAgdHJ5IHtcbiAgICBmaXJzdFRpbWVzdGFtcCA9IGF3YWl0IChcbiAgICAgIGF3YWl0IGZldGNoKGJ1aWxkVGltZXN0YW1wUGF0aCwgeyBjYWNoZTogXCJuby1zdG9yZVwiIH0pXG4gICAgKS5qc29uKCk7XG4gIH0gY2F0Y2ggKGVycm9yKSB7XG4gICAgY29uc29sZS5pbmZvKFxuICAgICAgXCJBdXRvLXJlbG9hZCBub3QgZW5hYmxlZCBiZWNhdXNlIGJ1aWxkIHRpbWVzdGFtcCBmaWxlIG5vdCBmb3VuZC5cIixcbiAgICAgIGVycm9yLFxuICAgICk7XG4gICAgcmV0dXJuO1xuICB9XG5cbiAgd2hpbGUgKHRydWUpIHtcbiAgICBsZXQgY3VycmVudFRpbWVzdGFtcDtcbiAgICB0cnkge1xuICAgICAgY3VycmVudFRpbWVzdGFtcCA9IGF3YWl0IChhd2FpdCBmZXRjaChidWlsZFRpbWVzdGFtcFBhdGgpKS5qc29uKCk7XG4gICAgfSBjYXRjaCAoZXJyb3IpIHtcbiAgICAgIGlmIChwcmV2aW91c1RpbWVzdGFtcCkge1xuICAgICAgICBjb25zb2xlLmRlYnVnKFxuICAgICAgICAgIFwiRmFpbGVkIHRvIGxvYWQgYnVpbGQgdGltZXN0YW1wIGZpbGU7IGEgbmV3IGJ1aWxkIG1heSBiZSBpbiBwcm9ncmVzcy5cIixcbiAgICAgICAgICBlcnJvcixcbiAgICAgICAgKTtcbiAgICAgIH1cbiAgICAgIC8vIE9uY2Ugd2Uga25vdyBhIGJ1aWxkIGlzIGluIHByb2dyZXNzLCBwb2xsIHJhcGlkbHkgZm9yIGl0cyBjb21wbGV0aW9uLlxuICAgICAgYXdhaXQgbmV3IFByb21pc2UocmVzb2x2ZSA9PiBzZXRUaW1lb3V0KHJlc29sdmUsIDI1MCkpO1xuICAgICAgcHJldmlvdXNUaW1lc3RhbXAgPSB1bmRlZmluZWQ7XG4gICAgICBjb250aW51ZTtcbiAgICB9XG5cbiAgICBpZiAoY3VycmVudFRpbWVzdGFtcCAhPT0gZmlyc3RUaW1lc3RhbXApIHtcbiAgICAgIGNvbnNvbGUuaW5mbyhcIk5ldyBidWlsZCB0aW1lc3RhbXAgZGV0ZWN0ZWQsIGF1dG8tcmVsb2FkaW5nLlwiKTtcbiAgICAgIGRvY3VtZW50LmxvY2F0aW9uLnJlbG9hZCgpO1xuICAgICAgYXdhaXQgbmV3IFByb21pc2UocmVzb2x2ZSA9PiBzZXRUaW1lb3V0KHJlc29sdmUsIDQwMDApKTtcbiAgICAgIGNvbnNvbGUuZXJyb3IoXCJEaWQgd2UgZmFpbCB0byByZWxvYWQ/XCIpO1xuICAgIH1cblxuICAgIC8vIEJ1aWxkIHVzdWFsbHkgdGFrZXMgbG9uZ2VyIHRoYW4gdGhpcyBzbyB3ZSBkb24ndCBuZWVkIHRvIHBvbGwgbW9yZSBvZnRlbi5cbiAgICBhd2FpdCBuZXcgUHJvbWlzZShyZXNvbHZlID0+IHNldFRpbWVvdXQocmVzb2x2ZSwgNDAwMCkpO1xuICAgIHByZXZpb3VzVGltZXN0YW1wID0gY3VycmVudFRpbWVzdGFtcDtcbiAgfVxufSkoKTtcbiJdfQ==