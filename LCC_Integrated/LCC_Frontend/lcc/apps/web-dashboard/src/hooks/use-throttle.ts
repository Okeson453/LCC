import { useEffect, useRef, useState } from 'react';

export function useThrottle<T>(value: T, intervalMs = 250): T {
  const [throttled, setThrottled] = useState(value);
  const lastUpdate = useRef<number>(0);

  useEffect(() => {
    const now = Date.now();
    if (now - lastUpdate.current >= intervalMs) {
      lastUpdate.current = now;
      setThrottled(value);
      return;
    }
    const id = setTimeout(() => {
      lastUpdate.current = Date.now();
      setThrottled(value);
    }, intervalMs - (now - lastUpdate.current));
    return () => clearTimeout(id);
  }, [value, intervalMs]);

  return throttled;
}
