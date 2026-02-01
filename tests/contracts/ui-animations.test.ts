/**
 * UI Animation Contract Tests
 * Issue #91 - STORY-UX-001
 *
 * Validates:
 * - I-UI-001: Reduced motion support
 * - I-UI-002: Consistent timing
 * - I-UI-003: Zero layout shift (CLS = 0)
 */

import { describe, it, expect, beforeEach, afterEach } from '@jest/globals';
import {
  ANIMATION_DURATIONS,
  EASING_FUNCTIONS,
  AnimationService,
  getAnimationService,
} from '../../web/src/animations';

describe('Contract: I-UI-001 - Reduced Motion Support', () => {
  let mockMediaQuery: {
    matches: boolean;
    addEventListener: jest.Mock;
    removeEventListener: jest.Mock;
  };
  let originalMatchMedia: typeof window.matchMedia;

  beforeEach(() => {
    // Mock matchMedia
    mockMediaQuery = {
      matches: false,
      addEventListener: jest.fn(),
      removeEventListener: jest.fn(),
    };

    originalMatchMedia = window.matchMedia;
    window.matchMedia = jest.fn(() => mockMediaQuery as any);
  });

  afterEach(() => {
    window.matchMedia = originalMatchMedia;
  });

  it('MUST detect prefers-reduced-motion setting', () => {
    mockMediaQuery.matches = true;
    const service = new AnimationService();

    expect(service.getReducedMotion()).toBe(true);
  });

  it('MUST return 0ms duration when reduced motion is enabled', () => {
    mockMediaQuery.matches = true;
    const service = new AnimationService();

    expect(service.getDuration('fast')).toBe(0);
    expect(service.getDuration('standard')).toBe(0);
    expect(service.getDuration('slow')).toBe(0);
  });

  it('MUST return normal duration when reduced motion is disabled', () => {
    mockMediaQuery.matches = false;
    const service = new AnimationService();

    expect(service.getDuration('fast')).toBe(150);
    expect(service.getDuration('standard')).toBe(300);
    expect(service.getDuration('slow')).toBe(500);
  });

  it('MUST listen for media query changes', () => {
    const service = new AnimationService();

    expect(mockMediaQuery.addEventListener).toHaveBeenCalledWith(
      'change',
      expect.any(Function)
    );
  });

  it('MUST animate with 0ms duration when reduced motion is enabled', () => {
    mockMediaQuery.matches = true;
    const service = new AnimationService();

    const element = document.createElement('div');
    const result = service.fadeIn(element);

    // Animation should complete immediately
    expect(result.animation.currentTime).toBe(0);
  });
});

describe('Contract: I-UI-002 - Consistent Timing', () => {
  it('MUST define fast animations as 150ms', () => {
    expect(ANIMATION_DURATIONS.fast).toBe(150);
  });

  it('MUST define standard animations as 300ms', () => {
    expect(ANIMATION_DURATIONS.standard).toBe(300);
  });

  it('MUST define slow animations as 500ms', () => {
    expect(ANIMATION_DURATIONS.slow).toBe(500);
  });

  it('MUST define very slow animations as 1000ms', () => {
    expect(ANIMATION_DURATIONS.verySlow).toBe(1000);
  });

  it('MUST define instant animations as 0ms', () => {
    expect(ANIMATION_DURATIONS.instant).toBe(0);
  });

  it('MUST provide easeInOut easing function', () => {
    expect(EASING_FUNCTIONS.easeInOut).toBe('cubic-bezier(0.4, 0, 0.2, 1)');
  });

  it('MUST provide easeOut easing function', () => {
    expect(EASING_FUNCTIONS.easeOut).toBe('cubic-bezier(0.0, 0, 0.2, 1)');
  });

  it('MUST provide easeIn easing function', () => {
    expect(EASING_FUNCTIONS.easeIn).toBe('cubic-bezier(0.4, 0, 1, 1)');
  });

  it('MUST provide sharp easing function', () => {
    expect(EASING_FUNCTIONS.sharp).toBe('cubic-bezier(0.4, 0, 0.6, 1)');
  });

  it('MUST use consistent timing from constants', () => {
    const service = getAnimationService();

    expect(service.getDuration('fast')).toBe(ANIMATION_DURATIONS.fast);
    expect(service.getDuration('standard')).toBe(ANIMATION_DURATIONS.standard);
    expect(service.getDuration('slow')).toBe(ANIMATION_DURATIONS.slow);
  });

  it('MUST use consistent easing from constants', () => {
    const service = getAnimationService();

    expect(service.getEasing('easeInOut')).toBe(EASING_FUNCTIONS.easeInOut);
    expect(service.getEasing('easeOut')).toBe(EASING_FUNCTIONS.easeOut);
    expect(service.getEasing('easeIn')).toBe(EASING_FUNCTIONS.easeIn);
  });
});

describe('Contract: I-UI-003 - Zero Layout Shift', () => {
  let service: AnimationService;
  let element: HTMLDivElement;

  beforeEach(() => {
    service = getAnimationService();
    element = document.createElement('div');
    document.body.appendChild(element);
  });

  afterEach(() => {
    document.body.removeChild(element);
  });

  it('MUST only animate transform and opacity properties', () => {
    const result = service.scaleIn(element);
    const effect = result.animation.effect as KeyframeEffect;
    const keyframes = effect?.getKeyframes() || [];

    // Check that keyframes only contain transform and opacity
    keyframes.forEach((frame: any) => {
      const allowedProps = ['transform', 'opacity', 'offset', 'easing', 'composite'];
      const frameProps = Object.keys(frame);

      frameProps.forEach((prop) => {
        expect(allowedProps).toContain(prop);
      });
    });
  });

  it('MUST use transform for scale animations', () => {
    const result = service.scaleIn(element);
    const effect = result.animation.effect as KeyframeEffect;
    const keyframes = effect?.getKeyframes() || [];

    const hasTransform = keyframes.some((frame: any) =>
      frame.transform && frame.transform.includes('scale')
    );

    expect(hasTransform).toBe(true);
  });

  it('MUST use transform for slide animations', () => {
    const result = service.slideUp(element);
    const effect = result.animation.effect as KeyframeEffect;
    const keyframes = effect?.getKeyframes() || [];

    const hasTranslate = keyframes.some((frame: any) =>
      frame.transform && frame.transform.includes('translate')
    );

    expect(hasTranslate).toBe(true);
  });

  it('MUST use opacity for fade animations', () => {
    const result = service.fadeIn(element);
    const effect = result.animation.effect as KeyframeEffect;
    const keyframes = effect?.getKeyframes() || [];

    const hasOpacity = keyframes.some((frame: any) =>
      frame.opacity !== undefined
    );

    expect(hasOpacity).toBe(true);
  });

  it('MUST NOT animate width, height, top, left, or margin', () => {
    const animations = [
      service.fadeIn(element),
      service.scaleIn(element),
      service.slideUp(element),
      service.pulse(element),
    ];

    animations.forEach((result) => {
      const effect = result.animation.effect as KeyframeEffect;
      const keyframes = effect?.getKeyframes() || [];
      keyframes.forEach((frame: any) => {
        expect(frame.width).toBeUndefined();
        expect(frame.height).toBeUndefined();
        expect(frame.top).toBeUndefined();
        expect(frame.left).toBeUndefined();
        expect(frame.margin).toBeUndefined();
      });
    });
  });
});

describe('AnimationService - General Functionality', () => {
  let service: AnimationService;
  let element: HTMLDivElement;

  beforeEach(() => {
    service = getAnimationService();
    element = document.createElement('div');
    document.body.appendChild(element);
  });

  afterEach(() => {
    document.body.removeChild(element);
  });

  it('should provide singleton instance', () => {
    const service1 = getAnimationService();
    const service2 = getAnimationService();

    expect(service1).toBe(service2);
  });

  it('should animate element with custom config', () => {
    const result = service.animate(element, {
      keyframes: [{ opacity: 0 }, { opacity: 1 }],
      duration: 300,
      easing: 'ease-in-out',
    });

    expect(result.animation).toBeDefined();
    expect(result.promise).toBeDefined();
  });

  it('should animate group of elements with stagger', () => {
    const elements = [
      document.createElement('div'),
      document.createElement('div'),
      document.createElement('div'),
    ];

    elements.forEach((el) => document.body.appendChild(el));

    const results = service.animateGroup(
      elements,
      {
        keyframes: [{ opacity: 0 }, { opacity: 1 }],
        duration: 300,
      },
      50
    );

    expect(results).toHaveLength(3);
    results.forEach((result) => {
      expect(result.animation).toBeDefined();
    });

    elements.forEach((el) => document.body.removeChild(el));
  });
});
