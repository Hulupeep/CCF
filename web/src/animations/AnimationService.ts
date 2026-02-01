/**
 * Animation Service
 * Issue #91 - STORY-UX-001
 *
 * Core animation utility service providing:
 * - Reduced motion detection (I-UI-001)
 * - Consistent timing access (I-UI-002)
 * - Web Animations API wrapper
 * - Group animation coordination
 */

import {
  ANIMATION_DURATIONS,
  EASING_FUNCTIONS,
  AnimationType,
  EasingType,
} from './constants';

export interface AnimationConfig {
  duration?: number;
  easing?: string;
  delay?: number;
  fill?: FillMode;
  iterations?: number;
  direction?: PlaybackDirection;
  keyframes: Keyframe[];
}

export interface AnimationResult {
  animation: Animation;
  promise: Promise<void>;
}

export class AnimationService {
  private reducedMotion: boolean;
  private mediaQuery: MediaQueryList | null;

  constructor() {
    // I-UI-001: Detect prefers-reduced-motion
    this.mediaQuery = window.matchMedia('(prefers-reduced-motion: reduce)');
    this.reducedMotion = this.mediaQuery.matches;

    // Listen for changes
    this.mediaQuery.addEventListener('change', (e) => {
      this.reducedMotion = e.matches;
    });
  }

  /**
   * Check if reduced motion is enabled
   * I-UI-001: Accessibility requirement
   */
  getReducedMotion(): boolean {
    return this.reducedMotion;
  }

  /**
   * Get duration for animation type
   * I-UI-002: Consistent timing
   * Returns 0 if reduced motion is enabled
   */
  getDuration(type: AnimationType): number {
    if (this.reducedMotion) {
      return 0;
    }
    return ANIMATION_DURATIONS[type];
  }

  /**
   * Get easing function
   * I-UI-002: Consistent easing
   */
  getEasing(type: EasingType): string {
    return EASING_FUNCTIONS[type];
  }

  /**
   * Animate a single element
   * I-UI-001: Respects reduced motion
   * I-UI-003: Only animates transform/opacity by convention
   *
   * @param element - DOM element to animate
   * @param config - Animation configuration
   * @returns Animation instance and completion promise
   */
  animate(element: Element, config: AnimationConfig): AnimationResult {
    const duration = this.reducedMotion ? 0 : (config.duration ?? ANIMATION_DURATIONS.standard);

    const animation = element.animate(config.keyframes, {
      duration,
      easing: config.easing ?? EASING_FUNCTIONS.easeInOut,
      delay: config.delay ?? 0,
      fill: config.fill ?? 'none',
      iterations: config.iterations ?? 1,
      direction: config.direction ?? 'normal',
    });

    const promise = animation.finished.catch(() => {
      // Animation was cancelled, which is fine
    }) as Promise<void>;

    return { animation, promise };
  }

  /**
   * Animate multiple elements with the same configuration
   * Useful for staggered animations
   *
   * @param elements - Array of DOM elements
   * @param config - Animation configuration
   * @param staggerDelay - Optional delay between each element (ms)
   * @returns Array of animation results
   */
  animateGroup(
    elements: Element[],
    config: AnimationConfig,
    staggerDelay: number = 0
  ): AnimationResult[] {
    return elements.map((element, index) => {
      const delay = (config.delay ?? 0) + (index * staggerDelay);
      return this.animate(element, { ...config, delay });
    });
  }

  /**
   * Create a fade in animation
   */
  fadeIn(element: Element, duration?: number): AnimationResult {
    return this.animate(element, {
      keyframes: [
        { opacity: 0 },
        { opacity: 1 },
      ],
      duration: duration ?? this.getDuration('standard'),
      easing: this.getEasing('easeInOut'),
      fill: 'forwards',
    });
  }

  /**
   * Create a fade out animation
   */
  fadeOut(element: Element, duration?: number): AnimationResult {
    return this.animate(element, {
      keyframes: [
        { opacity: 1 },
        { opacity: 0 },
      ],
      duration: duration ?? this.getDuration('standard'),
      easing: this.getEasing('easeInOut'),
      fill: 'forwards',
    });
  }

  /**
   * Create a scale in animation
   */
  scaleIn(element: Element, duration?: number): AnimationResult {
    return this.animate(element, {
      keyframes: [
        { opacity: 0, transform: 'scale(0.9)' },
        { opacity: 1, transform: 'scale(1)' },
      ],
      duration: duration ?? this.getDuration('standard'),
      easing: this.getEasing('easeOut'),
      fill: 'forwards',
    });
  }

  /**
   * Create a scale out animation
   */
  scaleOut(element: Element, duration?: number): AnimationResult {
    return this.animate(element, {
      keyframes: [
        { opacity: 1, transform: 'scale(1)' },
        { opacity: 0, transform: 'scale(0.9)' },
      ],
      duration: duration ?? this.getDuration('fast'),
      easing: this.getEasing('easeIn'),
      fill: 'forwards',
    });
  }

  /**
   * Create a slide up animation
   */
  slideUp(element: Element, duration?: number): AnimationResult {
    return this.animate(element, {
      keyframes: [
        { transform: 'translateY(100%)', opacity: 0 },
        { transform: 'translateY(0)', opacity: 1 },
      ],
      duration: duration ?? this.getDuration('standard'),
      easing: this.getEasing('easeOut'),
      fill: 'forwards',
    });
  }

  /**
   * Create a slide down animation
   */
  slideDown(element: Element, duration?: number): AnimationResult {
    return this.animate(element, {
      keyframes: [
        { transform: 'translateY(0)', opacity: 1 },
        { transform: 'translateY(100%)', opacity: 0 },
      ],
      duration: duration ?? this.getDuration('fast'),
      easing: this.getEasing('easeIn'),
      fill: 'forwards',
    });
  }

  /**
   * Create a pulse animation (infinite loop)
   */
  pulse(element: Element, duration?: number): AnimationResult {
    return this.animate(element, {
      keyframes: [
        { opacity: 0.6 },
        { opacity: 1 },
        { opacity: 0.6 },
      ],
      duration: duration ?? this.getDuration('verySlow'),
      easing: this.getEasing('easeInOut'),
      iterations: Infinity,
    });
  }
}

// Singleton instance
let animationService: AnimationService | null = null;

/**
 * Get the global animation service instance
 */
export function getAnimationService(): AnimationService {
  if (!animationService) {
    animationService = new AnimationService();
  }
  return animationService;
}
