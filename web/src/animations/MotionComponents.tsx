/**
 * Motion Components
 * Issue #91 - STORY-UX-001
 *
 * Framer Motion wrapper components with:
 * - Reduced motion support (I-UI-001)
 * - Consistent timing (I-UI-002)
 * - Zero layout shift (I-UI-003)
 */

import React from 'react';
import { motion, MotionProps, Variants } from 'framer-motion';
import { ANIMATION_DURATIONS, EASING_FUNCTIONS } from './constants';
import { useReducedMotion } from './hooks';

/**
 * Common animation variants
 */
export const fadeInVariants: Variants = {
  hidden: { opacity: 0 },
  visible: {
    opacity: 1,
    transition: {
      duration: ANIMATION_DURATIONS.standard / 1000,
      ease: [0.4, 0, 0.2, 1], // easeInOut
    },
  },
};

export const scaleInVariants: Variants = {
  hidden: { opacity: 0, scale: 0.9 },
  visible: {
    opacity: 1,
    scale: 1,
    transition: {
      duration: ANIMATION_DURATIONS.standard / 1000,
      ease: [0.0, 0, 0.2, 1], // easeOut
    },
  },
};

export const slideUpVariants: Variants = {
  hidden: { opacity: 0, y: '100%' },
  visible: {
    opacity: 1,
    y: 0,
    transition: {
      duration: ANIMATION_DURATIONS.standard / 1000,
      ease: [0.0, 0, 0.2, 1], // easeOut
    },
  },
};

export const slideDownVariants: Variants = {
  hidden: { opacity: 0, y: '-100%' },
  visible: {
    opacity: 1,
    y: 0,
    transition: {
      duration: ANIMATION_DURATIONS.standard / 1000,
      ease: [0.0, 0, 0.2, 1], // easeOut
    },
  },
};

/**
 * FadeIn Component
 * Fades in content on mount
 */
interface FadeInProps extends Omit<MotionProps, 'variants' | 'initial' | 'animate'> {
  children: React.ReactNode;
  delay?: number;
  duration?: number;
}

export const FadeIn: React.FC<FadeInProps> = ({
  children,
  delay = 0,
  duration = ANIMATION_DURATIONS.standard,
  ...props
}) => {
  const reducedMotion = useReducedMotion();

  return (
    <motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      transition={{
        duration: reducedMotion ? 0 : duration / 1000,
        delay: reducedMotion ? 0 : delay / 1000,
        ease: [0.4, 0, 0.2, 1], // easeInOut
      }}
      {...props}
    >
      {children}
    </motion.div>
  );
};

/**
 * ScaleIn Component
 * Scales and fades in content on mount
 */
interface ScaleInProps extends Omit<MotionProps, 'variants' | 'initial' | 'animate'> {
  children: React.ReactNode;
  delay?: number;
  duration?: number;
}

export const ScaleIn: React.FC<ScaleInProps> = ({
  children,
  delay = 0,
  duration = ANIMATION_DURATIONS.standard,
  ...props
}) => {
  const reducedMotion = useReducedMotion();

  return (
    <motion.div
      initial={{ opacity: 0, scale: 0.9 }}
      animate={{ opacity: 1, scale: 1 }}
      exit={{ opacity: 0, scale: 0.9 }}
      transition={{
        duration: reducedMotion ? 0 : duration / 1000,
        delay: reducedMotion ? 0 : delay / 1000,
        ease: [0.0, 0, 0.2, 1], // easeOut
      }}
      {...props}
    >
      {children}
    </motion.div>
  );
};

/**
 * SlideUp Component
 * Slides up and fades in from bottom
 */
interface SlideUpProps {
  children: React.ReactNode;
  delay?: number;
  duration?: number;
  className?: string;
  style?: React.CSSProperties;
  onClick?: (event: React.MouseEvent<HTMLDivElement>) => void;
  'data-testid'?: string;
}

export const SlideUp: React.FC<SlideUpProps> = ({
  children,
  delay = 0,
  duration = ANIMATION_DURATIONS.standard,
  className,
  style,
  onClick,
  'data-testid': dataTestId,
}) => {
  const reducedMotion = useReducedMotion();

  return (
    <motion.div
      className={className}
      style={style}
      onClick={onClick}
      data-testid={dataTestId}
      initial={{ opacity: 0, y: 50 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: 50 }}
      transition={{
        duration: reducedMotion ? 0 : duration / 1000,
        delay: reducedMotion ? 0 : delay / 1000,
        ease: [0.0, 0, 0.2, 1], // easeOut
      }}
    >
      {children}
    </motion.div>
  );
};

/**
 * AnimatedButton Component
 * Button with hover and tap animations
 */
interface AnimatedButtonProps {
  children: React.ReactNode;
  className?: string;
  onClick?: (event: React.MouseEvent<HTMLButtonElement>) => void;
  disabled?: boolean;
  type?: 'button' | 'submit' | 'reset';
  'data-testid'?: string;
}

export const AnimatedButton: React.FC<AnimatedButtonProps> = ({
  children,
  className = '',
  onClick,
  disabled,
  type = 'button',
  'data-testid': dataTestId,
}) => {
  const reducedMotion = useReducedMotion();

  return (
    <motion.button
      className={className}
      onClick={onClick}
      disabled={disabled}
      type={type}
      data-testid={dataTestId}
      whileHover={reducedMotion ? {} : { scale: 1.05 }}
      whileTap={reducedMotion ? {} : { scale: 0.95 }}
      transition={{
        duration: ANIMATION_DURATIONS.fast / 1000,
        ease: [0.4, 0, 0.2, 1], // easeInOut
      }}
    >
      {children}
    </motion.button>
  );
};

/**
 * Stagger Container
 * Animates children with stagger effect
 */
interface StaggerContainerProps {
  children: React.ReactNode;
  staggerDelay?: number;
  className?: string;
}

export const StaggerContainer: React.FC<StaggerContainerProps> = ({
  children,
  staggerDelay = 50,
  className = '',
}) => {
  const reducedMotion = useReducedMotion();

  return (
    <motion.div
      className={className}
      initial="hidden"
      animate="visible"
      variants={{
        visible: {
          transition: {
            staggerChildren: reducedMotion ? 0 : staggerDelay / 1000,
          },
        },
      }}
    >
      {children}
    </motion.div>
  );
};

/**
 * Stagger Item
 * Use inside StaggerContainer
 */
interface StaggerItemProps {
  children: React.ReactNode;
  className?: string;
}

export const StaggerItem: React.FC<StaggerItemProps> = ({ children, className = '' }) => {
  return (
    <motion.div
      className={className}
      variants={fadeInVariants}
    >
      {children}
    </motion.div>
  );
};

/**
 * Modal Backdrop
 * Animated backdrop for modals
 */
interface ModalBackdropProps {
  children: React.ReactNode;
  onClose?: () => void;
}

export const ModalBackdrop: React.FC<ModalBackdropProps> = ({ children, onClose }) => {
  const reducedMotion = useReducedMotion();

  return (
    <motion.div
      className="modal-backdrop"
      data-testid="modal-backdrop"
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      transition={{
        duration: reducedMotion ? 0 : ANIMATION_DURATIONS.standard / 1000,
      }}
      onClick={onClose}
    >
      {children}
    </motion.div>
  );
};

/**
 * Modal Content
 * Animated modal content
 */
interface ModalContentProps {
  children: React.ReactNode;
  className?: string;
}

export const ModalContent: React.FC<ModalContentProps> = ({ children, className = '' }) => {
  const reducedMotion = useReducedMotion();

  return (
    <motion.div
      className={`modal-content ${className}`}
      data-testid="modal-content"
      initial={{ opacity: 0, scale: 0.9 }}
      animate={{ opacity: 1, scale: 1 }}
      exit={{ opacity: 0, scale: 0.9 }}
      transition={{
        duration: reducedMotion ? 0 : ANIMATION_DURATIONS.standard / 1000,
        ease: [0.0, 0, 0.2, 1], // easeOut
      }}
      onClick={(e) => e.stopPropagation()}
    >
      {children}
    </motion.div>
  );
};

/**
 * Skeleton Loader
 * Animated loading placeholder
 */
interface SkeletonProps {
  width?: string | number;
  height?: string | number;
  className?: string;
}

export const Skeleton: React.FC<SkeletonProps> = ({
  width = '100%',
  height = 20,
  className = '',
}) => {
  const reducedMotion = useReducedMotion();

  return (
    <motion.div
      className={`skeleton ${className}`}
      data-testid="skeleton-loader"
      style={{ width, height }}
      animate={
        reducedMotion
          ? {}
          : {
              opacity: [0.6, 1, 0.6],
            }
      }
      transition={
        reducedMotion
          ? {}
          : {
              duration: ANIMATION_DURATIONS.verySlow / 1000,
              repeat: Infinity,
              ease: [0.4, 0, 0.2, 1], // easeInOut
            }
      }
    />
  );
};
