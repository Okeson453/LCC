import { useEffect, useRef, useState } from 'react';

export function useIntersectionObserver<T extends Element>(
  options: IntersectionObserverInit = { threshold: 0.1 },
): [React.RefObject<T>, boolean] {
  const ref = useRef<T>(null);
  const [intersecting, setIntersecting] = useState(false);

  useEffect(() => {
    if (!ref.current) return;
    if (typeof IntersectionObserver === 'undefined') {
      setIntersecting(true);
      return;
    }
    const observer = new IntersectionObserver(([entry]) => {
      setIntersecting(entry?.isIntersecting ?? false);
    }, options);
    observer.observe(ref.current);
    return () => observer.disconnect();
  }, [options]);

  return [ref, intersecting];
}
