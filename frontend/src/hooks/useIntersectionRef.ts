import { useRef, useCallback } from 'react';

/**
 * Returns a ref callback that triggers `onIntersect` when the attached element
 * enters the viewport. Automatically reconnects when dependencies change.
 *
 * @param onIntersect - Called when the observed element is visible.
 * @param enabled     - When false the observer is not attached (e.g. while loading).
 */
export function useIntersectionRef(
  onIntersect: () => void,
  enabled: boolean,
): (node: Element | null) => void {
  const observerRef = useRef<IntersectionObserver | null>(null);

  return useCallback(
    (node) => {
      if (!enabled) return;

      observerRef.current?.disconnect();

      if (!node) return;

      observerRef.current = new IntersectionObserver(([entry]) => {
        if (entry.isIntersecting) onIntersect();
      });

      observerRef.current.observe(node);
    },
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [enabled, onIntersect],
  );
}