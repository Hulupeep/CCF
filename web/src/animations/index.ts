/**
 * Animation System Exports
 * Issue #91 - STORY-UX-001
 *
 * Central export point for all animation utilities
 */

// Constants
export {
  ANIMATION_DURATIONS,
  EASING_FUNCTIONS,
  ANIMATION_KEYFRAMES,
  COMPONENT_ANIMATIONS,
  type AnimationType,
  type EasingType,
} from './constants';

// Service
export {
  AnimationService,
  getAnimationService,
  type AnimationConfig,
  type AnimationResult,
} from './AnimationService';

// Hooks
export {
  useReducedMotion,
  useAnimation,
  useFadeIn,
  useAnimationDuration,
} from './hooks';

// Motion Components
export {
  FadeIn,
  ScaleIn,
  SlideUp,
  AnimatedButton,
  StaggerContainer,
  StaggerItem,
  ModalBackdrop,
  ModalContent,
  Skeleton,
  fadeInVariants,
  scaleInVariants,
  slideUpVariants,
  slideDownVariants,
} from './MotionComponents';

// Toast
export {
  ToastProvider,
  useToast,
  showToast,
  setToastFunction,
  type Toast,
  type ToastType,
} from './Toast';
