import { useEffect, useState } from "react";

const tick = 30_000;

export function useNow() {
  const [now, setNow] = useState(Date.now);

  useEffect(() => {
    const timer = setInterval(() => {
      setNow(Date.now());
    }, tick);

    return () => {
      clearInterval(timer);
    };
  }, []);

  return now;
}
