function mockAnimation(): Animation {
  let cancelled = false;
  let finishHandler: Animation["onfinish"] = null;

  const animation = {
    currentTime: 0,
    effect: null,
    playState: "finished",
    cancel() {
      cancelled = true;
    },
    get onfinish() {
      return finishHandler;
    },
    set onfinish(handler) {
      finishHandler = handler;
      if (handler) {
        queueMicrotask(() => {
          if (!cancelled) {
            handler.call(
              animation,
              new Event("finish") as AnimationPlaybackEvent,
            );
          }
        });
      }
    },
  } as unknown as Animation;

  return animation;
}

if (!Element.prototype.animate) {
  Object.defineProperty(Element.prototype, "animate", {
    configurable: true,
    value: mockAnimation,
  });
}
