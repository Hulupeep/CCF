/**
 * Test Setup
 * Global configuration for Jest tests
 */

// Mock Web Animations API (not implemented in jsdom)
Element.prototype.animate = function(keyframes: Keyframe[], options?: KeyframeAnimationOptions): Animation {
  const animation: any = {
    currentTime: 0,
    playState: 'running',
    pending: false,
    effect: {
      getKeyframes: () => keyframes,
      getTiming: () => ({
        duration: typeof options === 'object' ? options.duration : 0,
        easing: typeof options === 'object' ? options.easing : 'linear',
      }),
    } as KeyframeEffect,
    play: jest.fn(),
    pause: jest.fn(),
    cancel: jest.fn(),
    finish: jest.fn(),
    reverse: jest.fn(),
    updatePlaybackRate: jest.fn(),
    persist: jest.fn(),
    commitStyles: jest.fn(),
    addEventListener: jest.fn(),
    removeEventListener: jest.fn(),
    dispatchEvent: jest.fn(),
  };

  // Add circular promises after object creation
  animation.finished = Promise.resolve(animation);
  animation.ready = Promise.resolve(animation);

  return animation;
};
