# Animation Guide

**Issue #91 - STORY-UX-001**

This guide explains how to use the animation system in mBot RuVector web dashboard.

## Table of Contents

- [Overview](#overview)
- [Animation Contracts](#animation-contracts)
- [Quick Start](#quick-start)
- [Animation Service](#animation-service)
- [React Hooks](#react-hooks)
- [Framer Motion Components](#framer-motion-components)
- [Toast Notifications](#toast-notifications)
- [Best Practices](#best-practices)
- [Performance Tips](#performance-tips)
- [Accessibility](#accessibility)

## Overview

The animation system provides:

- **Consistent timing** - Standard durations (150ms fast, 300ms standard, 500ms slow)
- **Smooth easing** - Material Design inspired curves
- **Reduced motion support** - Respects user accessibility preferences
- **Zero layout shift** - Only animates transform/opacity
- **60fps performance** - GPU-accelerated animations

## Animation Contracts

All animations must comply with these contracts:

### I-UI-001: Reduced Motion Support

**MUST** respect `prefers-reduced-motion` accessibility setting.

```typescript
import { useReducedMotion } from '@/animations';

const reducedMotion = useReducedMotion();

// Animation duration is 0ms when reduced motion is enabled
const duration = reducedMotion ? 0 : 300;
```

### I-UI-002: Consistent Timing

**MUST** use standard timing constants.

```typescript
import { ANIMATION_DURATIONS } from '@/animations';

const duration = ANIMATION_DURATIONS.standard; // 300ms
```

| Type | Duration | Use Case |
|------|----------|----------|
| instant | 0ms | Slider drag feedback |
| fast | 150ms | Button interactions |
| standard | 300ms | Most transitions |
| slow | 500ms | Large content changes |
| verySlow | 1000ms | Skeleton pulse |

### I-UI-003: Zero Layout Shift

**MUST** only animate `transform` and `opacity` properties.

```typescript
// ✅ CORRECT - No layout shift
.animate({
  opacity: [0, 1],
  transform: ['scale(0.9)', 'scale(1)']
})

// ❌ WRONG - Causes layout shift
.animate({
  width: ['100px', '200px'],
  height: ['50px', '100px']
})
```

## Quick Start

### 1. Using Animation Service (Web Animations API)

```typescript
import { getAnimationService } from '@/animations';

const service = getAnimationService();
const element = document.querySelector('.my-element');

// Fade in
service.fadeIn(element);

// Scale in
service.scaleIn(element);

// Slide up
service.slideUp(element);

// Custom animation
service.animate(element, {
  keyframes: [
    { opacity: 0, transform: 'translateY(20px)' },
    { opacity: 1, transform: 'translateY(0)' }
  ],
  duration: service.getDuration('standard'),
  easing: service.getEasing('easeOut')
});
```

### 2. Using React Hooks

```typescript
import { useAnimation } from '@/animations';

function MyComponent() {
  const { animate, isAnimating } = useAnimation();
  const ref = useRef<HTMLDivElement>(null);

  const handleClick = () => {
    if (ref.current) {
      animate(ref.current, {
        keyframes: [
          { transform: 'scale(1)' },
          { transform: 'scale(1.1)' },
          { transform: 'scale(1)' }
        ],
        duration: 300
      });
    }
  };

  return <div ref={ref} onClick={handleClick}>Click me</div>;
}
```

### 3. Using Framer Motion Components

```typescript
import { FadeIn, ScaleIn, SlideUp, AnimatedButton } from '@/animations';

function MyComponent() {
  return (
    <>
      <FadeIn>
        <h1>This fades in on mount</h1>
      </FadeIn>

      <ScaleIn delay={100}>
        <p>This scales in after 100ms</p>
      </ScaleIn>

      <SlideUp>
        <div>This slides up from bottom</div>
      </SlideUp>

      <AnimatedButton onClick={() => alert('Clicked!')}>
        Click me (with hover/tap animations)
      </AnimatedButton>
    </>
  );
}
```

## Animation Service

The `AnimationService` class wraps the Web Animations API with:

- Reduced motion detection
- Consistent timing
- Helper methods for common animations

### Methods

```typescript
// Get singleton instance
const service = getAnimationService();

// Check reduced motion
const reducedMotion = service.getReducedMotion(); // boolean

// Get timing
const duration = service.getDuration('standard'); // 300

// Get easing
const easing = service.getEasing('easeOut'); // 'cubic-bezier(...)'

// Animate single element
const { animation, promise } = service.fadeIn(element);

// Animate group with stagger
const results = service.animateGroup(
  [el1, el2, el3],
  { keyframes: [...], duration: 300 },
  50 // stagger delay
);
```

## React Hooks

### `useReducedMotion()`

Detect user's motion preference.

```typescript
const reducedMotion = useReducedMotion();

<motion.div
  animate={{ opacity: 1 }}
  transition={{ duration: reducedMotion ? 0 : 0.3 }}
/>
```

### `useAnimation()`

Wrapper for animating elements.

```typescript
const { animate, isAnimating, cancel } = useAnimation();

animate(elementRef.current, {
  keyframes: [{ opacity: 0 }, { opacity: 1 }],
  duration: 300
});
```

### `useFadeIn()`

Auto fade-in on mount.

```typescript
const ref = useFadeIn();

return <div ref={ref}>Fades in automatically</div>;
```

### `useAnimationDuration()`

Get timing constants.

```typescript
const { fast, standard, slow } = useAnimationDuration();

// Use in Framer Motion
<motion.div transition={{ duration: standard / 1000 }} />
```

## Framer Motion Components

Pre-built animated components.

### `<FadeIn>`

Fades in content on mount.

```typescript
<FadeIn delay={0} duration={300}>
  <div>Content</div>
</FadeIn>
```

### `<ScaleIn>`

Scales and fades in.

```typescript
<ScaleIn>
  <div>Content</div>
</ScaleIn>
```

### `<SlideUp>`

Slides up from bottom.

```typescript
<SlideUp>
  <div>Content</div>
</SlideUp>
```

### `<AnimatedButton>`

Button with hover/tap animations.

```typescript
<AnimatedButton onClick={handleClick}>
  Click me
</AnimatedButton>
```

### `<StaggerContainer>` + `<StaggerItem>`

Stagger animations for lists.

```typescript
<StaggerContainer staggerDelay={50}>
  {items.map(item => (
    <StaggerItem key={item.id}>
      <div>{item.name}</div>
    </StaggerItem>
  ))}
</StaggerContainer>
```

### `<ModalBackdrop>` + `<ModalContent>`

Animated modal.

```typescript
<AnimatePresence>
  {isOpen && (
    <ModalBackdrop onClose={handleClose}>
      <ModalContent>
        <h2>Modal Title</h2>
        <p>Modal content</p>
      </ModalContent>
    </ModalBackdrop>
  )}
</AnimatePresence>
```

### `<Skeleton>`

Loading placeholder.

```typescript
<Skeleton width="100%" height={20} />
<Skeleton width="80%" height={20} />
<Skeleton width="60%" height={20} />
```

## Toast Notifications

System-wide toast notifications.

### Setup

Wrap your app with `ToastProvider`:

```typescript
import { ToastProvider } from '@/animations';

function App() {
  return (
    <ToastProvider maxToasts={5}>
      <YourApp />
    </ToastProvider>
  );
}
```

### Usage

```typescript
import { useToast } from '@/animations';

function MyComponent() {
  const { addToast } = useToast();

  const handleSuccess = () => {
    addToast('Operation completed!', 'success', 3000);
  };

  const handleError = () => {
    addToast('Something went wrong', 'error', 5000);
  };

  return (
    <>
      <button onClick={handleSuccess}>Success</button>
      <button onClick={handleError}>Error</button>
    </>
  );
}
```

### Toast Types

- `success` - Green, for successful operations
- `error` - Red, for errors
- `warning` - Orange, for warnings
- `info` - Blue, for informational messages

## Best Practices

### 1. Always Use Constants

```typescript
// ✅ CORRECT
import { ANIMATION_DURATIONS } from '@/animations';
const duration = ANIMATION_DURATIONS.standard;

// ❌ WRONG
const duration = 300; // Hardcoded
```

### 2. Respect Reduced Motion

```typescript
// ✅ CORRECT
const reducedMotion = useReducedMotion();
const duration = reducedMotion ? 0 : 300;

// ❌ WRONG
const duration = 300; // Ignores user preference
```

### 3. Only Animate Transform/Opacity

```typescript
// ✅ CORRECT - No layout shift
animate({
  transform: 'scale(1.05)',
  opacity: 0.8
})

// ❌ WRONG - Causes layout shift
animate({
  width: '200px',
  height: '100px'
})
```

### 4. Use GPU Acceleration

```typescript
// ✅ CORRECT
.animated-element {
  will-change: transform, opacity;
}

// Remove after animation
.animation-complete {
  will-change: auto;
}
```

### 5. Provide Instant Feedback

For interactive elements (sliders, buttons), provide feedback <16ms:

```typescript
// Instant visual update during drag
const handleDrag = (value: number) => {
  requestAnimationFrame(() => {
    updateValue(value);
  });
};
```

## Performance Tips

### 1. Limit Animated Elements

Animate only what's visible on screen.

### 2. Use `will-change` Sparingly

Only use on elements that will definitely animate.

### 3. Batch Animations

Use `animateGroup()` for multiple elements:

```typescript
service.animateGroup(elements, config, staggerDelay);
```

### 4. Cancel Animations

Clean up when component unmounts:

```typescript
useEffect(() => {
  const { animation } = service.fadeIn(element);

  return () => {
    animation.cancel();
  };
}, []);
```

### 5. Use `AnimatePresence`

For enter/exit animations in React:

```typescript
<AnimatePresence>
  {isVisible && (
    <motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
    />
  )}
</AnimatePresence>
```

## Accessibility

### Reduced Motion

Always check and respect `prefers-reduced-motion`:

```typescript
const reducedMotion = useReducedMotion();

if (reducedMotion) {
  // Use instant transitions
  duration = 0;
} else {
  // Use normal animations
  duration = 300;
}
```

### Focus States

Ensure focus states are visible:

```css
*:focus-visible {
  outline: 2px solid #3b82f6;
  outline-offset: 2px;
}
```

### Keyboard Navigation

Make sure animations don't interfere with keyboard navigation.

## Testing

### Contract Tests

Run contract tests to ensure compliance:

```bash
npm test tests/contracts/ui-animations.test.ts
```

### Journey Tests

Run E2E journey tests:

```bash
npm run test:journeys tests/journeys/ui-polish.journey.spec.ts
```

### Visual Regression

Use Percy or Chromatic for visual regression testing.

## Troubleshooting

### Animations Not Working

1. Check if reduced motion is enabled
2. Verify element is mounted in DOM
3. Check console for errors
4. Ensure Framer Motion is installed

### Performance Issues

1. Limit number of animated elements
2. Remove `will-change` after animation
3. Use transform/opacity only
4. Check for forced synchronous layouts

### Layout Shift

1. Pre-allocate space for elements
2. Only animate transform/opacity
3. Use fixed dimensions where possible
4. Test with Lighthouse CLS metric

## Resources

- [Web Animations API](https://developer.mozilla.org/en-US/docs/Web/API/Web_Animations_API)
- [Framer Motion Docs](https://www.framer.com/motion/)
- [Material Design Motion](https://material.io/design/motion/)
- [Reduced Motion](https://web.dev/prefers-reduced-motion/)

## Support

For issues or questions:

- Check issue #91 for context
- Review contract tests for expected behavior
- Consult this guide for usage examples
