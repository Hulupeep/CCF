/**
 * Toast Notification Component
 * Issue #91 - STORY-UX-001
 *
 * Animated toast notifications with:
 * - Slide up entrance (300ms)
 * - Slide down exit (150ms)
 * - Auto-dismiss after 3 seconds
 * - Reduced motion support
 */

import React, { useEffect, useState, createContext, useContext } from 'react';
import { AnimatePresence } from 'framer-motion';
import { SlideUp } from './MotionComponents';
import { ANIMATION_DURATIONS } from './constants';

export type ToastType = 'success' | 'error' | 'info' | 'warning';

export interface Toast {
  id: string;
  message: string;
  type: ToastType;
  duration?: number;
}

interface ToastContextValue {
  addToast: (message: string, type?: ToastType, duration?: number) => void;
  removeToast: (id: string) => void;
}

const ToastContext = createContext<ToastContextValue | null>(null);

/**
 * Hook to access toast functions
 */
export function useToast() {
  const context = useContext(ToastContext);
  if (!context) {
    throw new Error('useToast must be used within ToastProvider');
  }
  return context;
}

/**
 * Toast Provider
 * Wrap your app with this to enable toasts
 */
interface ToastProviderProps {
  children: React.ReactNode;
  maxToasts?: number;
}

export const ToastProvider: React.FC<ToastProviderProps> = ({
  children,
  maxToasts = 5,
}) => {
  const [toasts, setToasts] = useState<Toast[]>([]);

  const addToast = (message: string, type: ToastType = 'info', duration: number = 3000) => {
    const id = `toast-${Date.now()}-${Math.random()}`;
    const newToast: Toast = { id, message, type, duration };

    setToasts((prev) => {
      const updated = [...prev, newToast];
      // Limit number of toasts
      return updated.slice(-maxToasts);
    });
  };

  const removeToast = (id: string) => {
    setToasts((prev) => prev.filter((toast) => toast.id !== id));
  };

  return (
    <ToastContext.Provider value={{ addToast, removeToast }}>
      {children}
      <ToastContainer toasts={toasts} onRemove={removeToast} />
    </ToastContext.Provider>
  );
};

/**
 * Toast Container
 * Renders all active toasts
 */
interface ToastContainerProps {
  toasts: Toast[];
  onRemove: (id: string) => void;
}

const ToastContainer: React.FC<ToastContainerProps> = ({ toasts, onRemove }) => {
  return (
    <div
      className="toast-container"
      data-testid="toast-container"
      style={{
        position: 'fixed',
        bottom: 20,
        right: 20,
        zIndex: 9999,
        display: 'flex',
        flexDirection: 'column',
        gap: 10,
        pointerEvents: 'none',
      }}
    >
      <AnimatePresence>
        {toasts.map((toast) => (
          <ToastItem key={toast.id} toast={toast} onRemove={onRemove} />
        ))}
      </AnimatePresence>
    </div>
  );
};

/**
 * Individual Toast Item
 */
interface ToastItemProps {
  toast: Toast;
  onRemove: (id: string) => void;
}

const ToastItem: React.FC<ToastItemProps> = ({ toast, onRemove }) => {
  useEffect(() => {
    if (toast.duration && toast.duration > 0) {
      const timer = setTimeout(() => {
        onRemove(toast.id);
      }, toast.duration);

      return () => clearTimeout(timer);
    }
  }, [toast.id, toast.duration, onRemove]);

  const getTypeStyles = (type: ToastType): React.CSSProperties => {
    const baseStyles: React.CSSProperties = {
      padding: '12px 20px',
      borderRadius: 8,
      color: 'white',
      fontWeight: 500,
      boxShadow: '0 4px 12px rgba(0, 0, 0, 0.15)',
      pointerEvents: 'auto',
      cursor: 'pointer',
      minWidth: 250,
      maxWidth: 400,
    };

    switch (type) {
      case 'success':
        return { ...baseStyles, backgroundColor: '#10b981' };
      case 'error':
        return { ...baseStyles, backgroundColor: '#ef4444' };
      case 'warning':
        return { ...baseStyles, backgroundColor: '#f59e0b' };
      case 'info':
      default:
        return { ...baseStyles, backgroundColor: '#3b82f6' };
    }
  };

  return (
    <SlideUp
      data-testid={`toast-${toast.id}`}
      style={getTypeStyles(toast.type)}
      onClick={() => onRemove(toast.id)}
      duration={ANIMATION_DURATIONS.standard}
    >
      <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
        <span style={{ flex: 1 }}>{toast.message}</span>
        <button
          onClick={(e) => {
            e.stopPropagation();
            onRemove(toast.id);
          }}
          style={{
            background: 'none',
            border: 'none',
            color: 'white',
            cursor: 'pointer',
            fontSize: 18,
            padding: 0,
            opacity: 0.8,
          }}
          aria-label="Close toast"
        >
          ×
        </button>
      </div>
    </SlideUp>
  );
};

/**
 * Standalone toast function (without provider)
 * Use when you can't wrap with ToastProvider
 */
let toastQueue: ((message: string, type?: ToastType) => void) | null = null;

export function setToastFunction(fn: (message: string, type?: ToastType) => void) {
  toastQueue = fn;
}

export function showToast(message: string, type: ToastType = 'info') {
  if (toastQueue) {
    toastQueue(message, type);
  } else {
    console.warn('Toast system not initialized. Wrap app with ToastProvider.');
  }
}
