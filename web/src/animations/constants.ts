/**
 * Animation Constants
 * Issue #91 - STORY-UX-001
 *
 * Defines timing and easing functions for consistent animations.
 * Implements I-UI-002: Consistent animation timing across the app.
 */

export type AnimationType = 'instant' | 'fast' | 'standard' | 'slow' | 'verySlow';
export type EasingType = 'easeInOut' | 'easeOut' | 'easeIn' | 'sharp';

/**
 * Standard animation durations (milliseconds)
 * I-UI-002: Consistent timing system
 */
export const ANIMATION_DURATIONS: Record<AnimationType, number> = {
  instant: 0,
  fast: 150,
  standard: 300,
  slow: 500,
  verySlow: 1000,
};

/**
 * Standard easing functions
 * Material Design inspired cubic-bezier curves
 */
export const EASING_FUNCTIONS: Record<EasingType, string> = {
  easeInOut: 'cubic-bezier(0.4, 0, 0.2, 1)', // Standard - most transitions
  easeOut: 'cubic-bezier(0.0, 0, 0.2, 1)', // Entrances - decelerating
  easeIn: 'cubic-bezier(0.4, 0, 1, 1)', // Exits - accelerating
  sharp: 'cubic-bezier(0.4, 0, 0.6, 1)', // Attention-grabbing
};

/**
 * Pre-defined animation keyframe sets
 * I-UI-003: Only use transform/opacity to prevent layout shift
 */
export const ANIMATION_KEYFRAMES = {
  fadeIn: [
    { opacity: 0 },
    { opacity: 1 },
  ],

  fadeOut: [
    { opacity: 1 },
    { opacity: 0 },
  ],

  scaleIn: [
    { opacity: 0, transform: 'scale(0.9)' },
    { opacity: 1, transform: 'scale(1)' },
  ],

  scaleOut: [
    { opacity: 1, transform: 'scale(1)' },
    { opacity: 0, transform: 'scale(0.9)' },
  ],

  slideUp: [
    { transform: 'translateY(100%)', opacity: 0 },
    { transform: 'translateY(0)', opacity: 1 },
  ],

  slideDown: [
    { transform: 'translateY(0)', opacity: 1 },
    { transform: 'translateY(100%)', opacity: 0 },
  ],

  slideLeft: [
    { transform: 'translateX(100%)', opacity: 0 },
    { transform: 'translateX(0)', opacity: 1 },
  ],

  slideRight: [
    { transform: 'translateX(0)', opacity: 1 },
    { transform: 'translateX(100%)', opacity: 0 },
  ],

  pulse: [
    { opacity: 0.6 },
    { opacity: 1 },
    { opacity: 0.6 },
  ],

  scaleButton: [
    { transform: 'scale(1)' },
    { transform: 'scale(0.95)' },
  ],

  scaleButtonHover: [
    { transform: 'scale(1)' },
    { transform: 'scale(1.05)' },
  ],
} as const;

/**
 * Component-specific animation configurations
 */
export const COMPONENT_ANIMATIONS = {
  slider: {
    duration: ANIMATION_DURATIONS.instant,
    easing: EASING_FUNCTIONS.easeOut,
  },

  button: {
    click: {
      duration: ANIMATION_DURATIONS.fast,
      easing: EASING_FUNCTIONS.easeInOut,
      keyframes: ANIMATION_KEYFRAMES.scaleButton,
    },
    hover: {
      duration: ANIMATION_DURATIONS.fast,
      easing: EASING_FUNCTIONS.easeOut,
      keyframes: ANIMATION_KEYFRAMES.scaleButtonHover,
    },
  },

  modal: {
    backdrop: {
      duration: ANIMATION_DURATIONS.standard,
      easing: EASING_FUNCTIONS.easeInOut,
      keyframes: ANIMATION_KEYFRAMES.fadeIn,
    },
    content: {
      duration: ANIMATION_DURATIONS.standard,
      easing: EASING_FUNCTIONS.easeOut,
      keyframes: ANIMATION_KEYFRAMES.scaleIn,
    },
  },

  toast: {
    enter: {
      duration: ANIMATION_DURATIONS.standard,
      easing: EASING_FUNCTIONS.easeOut,
      keyframes: ANIMATION_KEYFRAMES.slideUp,
    },
    exit: {
      duration: ANIMATION_DURATIONS.fast,
      easing: EASING_FUNCTIONS.easeIn,
      keyframes: ANIMATION_KEYFRAMES.slideDown,
    },
  },

  page: {
    transition: {
      duration: ANIMATION_DURATIONS.standard,
      easing: EASING_FUNCTIONS.easeInOut,
      keyframes: ANIMATION_KEYFRAMES.fadeIn,
    },
  },

  skeleton: {
    pulse: {
      duration: ANIMATION_DURATIONS.verySlow,
      easing: EASING_FUNCTIONS.easeInOut,
      keyframes: ANIMATION_KEYFRAMES.pulse,
      iterations: Infinity,
    },
  },
} as const;
