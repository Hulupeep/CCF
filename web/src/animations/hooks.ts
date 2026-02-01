/**
 * Animation Hooks
 * Issue #91 - STORY-UX-001
 *
 * React hooks for animation utilities:
 * - useReducedMotion - Detect accessibility preference
 * - useAnimation - Wrapper for Web Animations API
 * - useFadeIn - Automatic fade in on mount
 */

import { useEffect, useState, useRef, useCallback } from 'react';
import { getAnimationService } from './AnimationService';
import type { AnimationConfig, AnimationResult } from './AnimationService';

/**
 * Hook to detect reduced motion preference
 * I-UI-001: Accessibility requirement
 *
 * @returns true if user prefers reduced motion
 */
export function useReducedMotion(): boolean {
  const [reducedMotion, setReducedMotion] = useState(false);

  useEffect(() => {
    const service = getAnimationService();
    setReducedMotion(service.getReducedMotion());

    // Listen for changes
    const mediaQuery = window.matchMedia('(prefers-reduced-motion: reduce)');
    const handler = (e: MediaQueryListEvent) => {
      setReducedMotion(e.matches);
    };

    mediaQuery.addEventListener('change', handler);
    return () => mediaQuery.removeEventListener('change', handler);
  }, []);

  return reducedMotion;
}

/**
 * Hook to animate an element
 *
 * @returns Animate function and current animation state
 */
export function useAnimation() {
  const serviceRef = useRef(getAnimationService());
  const [isAnimating, setIsAnimating] = useState(false);
  const currentAnimationRef = useRef<Animation | null>(null);

  const animate = useCallback(
    (element: Element | null, config: AnimationConfig): AnimationResult | null => {
      if (!element) return null;

      // Cancel current animation if any
      if (currentAnimationRef.current) {
        currentAnimationRef.current.cancel();
      }

      setIsAnimating(true);
      const result = serviceRef.current.animate(element, config);

      currentAnimationRef.current = result.animation;

      result.promise.then(() => {
        setIsAnimating(false);
        currentAnimationRef.current = null;
      });

      return result;
    },
    []
  );

  const cancel = useCallback(() => {
    if (currentAnimationRef.current) {
      currentAnimationRef.current.cancel();
      setIsAnimating(false);
      currentAnimationRef.current = null;
    }
  }, []);

  return { animate, isAnimating, cancel };
}

/**
 * Hook to automatically fade in an element on mount
 *
 * @param deps - Dependencies to trigger re-animation
 */
export function useFadeIn(deps: React.DependencyList = []) {
  const ref = useRef<HTMLDivElement>(null);
  const serviceRef = useRef(getAnimationService());

  useEffect(() => {
    if (ref.current) {
      serviceRef.current.fadeIn(ref.current);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, deps);

  return ref;
}

/**
 * Hook to get animation durations
 * Useful for coordinating animations
 */
export function useAnimationDuration() {
  const serviceRef = useRef(getAnimationService());

  return {
    fast: serviceRef.current.getDuration('fast'),
    standard: serviceRef.current.getDuration('standard'),
    slow: serviceRef.current.getDuration('slow'),
    verySlow: serviceRef.current.getDuration('verySlow'),
  };
}
