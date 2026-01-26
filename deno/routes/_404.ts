export const handler = () =>
  new Response(
    `<!doctype html><style>head,title{display:flex}html{` +
      `background:#420;color:#FFF;font:bold clamp(16px,25vh,10vw)sans-serif;` +
      `padding:clamp(8px,10vh,5vw);user-select:none;-webkit-text-fill-color:` +
      `rgba(255,255,255,0.125);-webkit-text-stroke:max(2px,min(0.30vw))` +
      `}</style><title>404: Not Found`,
    { status: 404, headers: { "content-type": "text/html;charset=utf-8" } },
  );
