import { useEffect, useRef } from 'react';

export function useDidMount(): boolean {
  const ref = useRef(false);
  useEffect(() => {
    ref.current = true;
  }, []);
  return ref.current;
}
