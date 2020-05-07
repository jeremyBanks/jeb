describe("", async () => {
  const { localKey } = await import("./local-key.js");

  it("is alive!", async () => {
    expect(localKey({ type: "game" })).toMatchSnapshot();
  });
});
