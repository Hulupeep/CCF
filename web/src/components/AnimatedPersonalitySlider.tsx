/**
 * Animated Personality Slider
 * Issue #91 - STORY-UX-001
 *
 * Smooth slider with:
 * - Instant visual feedback (<16ms)
 * - Smooth easing (ease-out)
 * - Value label animation (150ms)
 * - No layout shift (I-UI-003)
 */

import React, { useState, useEffect, useRef } from 'react';
import { motion, useMotionValue, useTransform, animate } from 'framer-motion';
import { useReducedMotion } from '../animations';
import { ANIMATION_DURATIONS, EASING_FUNCTIONS } from '../animations/constants';

interface AnimatedSliderProps {
  label: string;
  value: number;
  previewValue?: number;
  min?: number;
  max?: number;
  step?: number;
  onChange: (value: number) => void;
  onChangeComplete?: () => void;
  disabled?: boolean;
  description?: string;
  testId?: string;
}

export const AnimatedPersonalitySlider: React.FC<AnimatedSliderProps> = ({
  label,
  value,
  previewValue,
  min = 0,
  max = 1,
  step = 0.01,
  onChange,
  onChangeComplete,
  disabled = false,
  description,
  testId,
}) => {
  const [isDragging, setIsDragging] = useState(false);
  const reducedMotion = useReducedMotion();
  const motionValue = useMotionValue(value);
  const prevValueRef = useRef(value);

  // Display value (preview during hover, actual value during drag)
  const displayValue = previewValue !== undefined && !isDragging ? previewValue : value;

  // Animate value changes (except during drag for instant feedback)
  useEffect(() => {
    if (!isDragging) {
      const duration = reducedMotion ? 0 : ANIMATION_DURATIONS.fast / 1000;
      animate(motionValue, value, { duration, ease: EASING_FUNCTIONS.easeOut });
      prevValueRef.current = value;
    }
  }, [value, isDragging, reducedMotion, motionValue]);

  // Transform motion value to percentage for visual indicator
  const percentage = useTransform(motionValue, [min, max], [0, 100]);

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const newValue = parseFloat(e.target.value);
    onChange(newValue);

    // Update motion value instantly during drag for immediate feedback (I-UI-001: <16ms)
    motionValue.set(newValue);
  };

  const handleMouseDown = () => {
    setIsDragging(true);
  };

  const handleMouseUp = () => {
    setIsDragging(false);
    onChangeComplete?.();
  };

  const handleTouchStart = () => {
    setIsDragging(true);
  };

  const handleTouchEnd = () => {
    setIsDragging(false);
    onChangeComplete?.();
  };

  return (
    <div
      className="animated-slider"
      data-testid={testId || `animated-slider-${label.toLowerCase().replace(/\s+/g, '-')}`}
      style={{
        marginBottom: 20,
        opacity: disabled ? 0.5 : 1,
        transition: `opacity ${ANIMATION_DURATIONS.fast}ms`,
      }}
    >
      {/* Header with label and animated value */}
      <div
        style={{
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center',
          marginBottom: 8,
        }}
      >
        <label
          style={{
            fontWeight: 500,
            fontSize: 14,
            color: '#374151',
          }}
        >
          {label}
        </label>

        {/* Animated value display */}
        <motion.span
          key={displayValue} // Key change triggers animation
          initial={reducedMotion ? {} : { scale: 1.2, opacity: 0 }}
          animate={{ scale: 1, opacity: 1 }}
          transition={{
            duration: reducedMotion ? 0 : ANIMATION_DURATIONS.fast / 1000,
            ease: EASING_FUNCTIONS.easeInOut,
          }}
          style={{
            fontWeight: 600,
            fontSize: 14,
            color: isDragging ? '#3b82f6' : '#6b7280',
            minWidth: 45,
            textAlign: 'right',
          }}
          data-testid={`${testId || label}-value`}
        >
          {displayValue.toFixed(2)}
        </motion.span>
      </div>

      {/* Description */}
      {description && (
        <div
          style={{
            fontSize: 12,
            color: '#6b7280',
            marginBottom: 8,
          }}
        >
          {description}
        </div>
      )}

      {/* Slider track container */}
      <div
        style={{
          position: 'relative',
          height: 8,
          marginBottom: 4,
        }}
      >
        {/* Background track */}
        <div
          style={{
            position: 'absolute',
            width: '100%',
            height: '100%',
            backgroundColor: '#e5e7eb',
            borderRadius: 4,
          }}
        />

        {/* Filled track (I-UI-003: transform only, no layout shift) */}
        <motion.div
          style={{
            position: 'absolute',
            height: '100%',
            backgroundColor: disabled ? '#9ca3af' : '#3b82f6',
            borderRadius: 4,
            scaleX: percentage.get() / 100,
            transformOrigin: 'left',
            transition: isDragging
              ? 'none'
              : `transform ${ANIMATION_DURATIONS.fast}ms ${EASING_FUNCTIONS.easeOut}`,
          }}
        />

        {/* Preview indicator (when hovering preset) */}
        {previewValue !== undefined && !isDragging && (
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 0.5 }}
            exit={{ opacity: 0 }}
            style={{
              position: 'absolute',
              height: '100%',
              backgroundColor: '#f59e0b',
              borderRadius: 4,
              width: `${((previewValue - min) / (max - min)) * 100}%`,
            }}
          />
        )}
      </div>

      {/* Actual input slider */}
      <input
        type="range"
        min={min}
        max={max}
        step={step}
        value={value}
        onChange={handleChange}
        onMouseDown={handleMouseDown}
        onMouseUp={handleMouseUp}
        onTouchStart={handleTouchStart}
        onTouchEnd={handleTouchEnd}
        disabled={disabled}
        data-testid={`${testId || label}-input`}
        style={{
          width: '100%',
          height: 8,
          opacity: 0,
          cursor: disabled ? 'not-allowed' : 'pointer',
          position: 'relative',
        }}
      />
    </div>
  );
};
