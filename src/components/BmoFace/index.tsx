import { useCallback, useEffect, useRef, useState } from "react";
import { useBmoStore } from "../../store";
import { FACES } from "./faces";

function randomFrom<T>(arr: readonly T[]): T {
  return arr[Math.floor(Math.random() * arr.length)];
}

const ANGRY_CLICK_THRESHOLD = 3;
const HAPPY_DURATION_MS = 2000;
const ANGRY_DURATION_MS = 1500;

export function BmoFace() {
  const isCollapsed = useBmoStore((s) => s.isCollapsed);

  const [currentFace, setCurrentFace] = useState<string>(randomFrom(FACES.idle));
  const seenIdle = useRef<Set<number>>(new Set());
  const clickCount = useRef(0);
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const clearPending = useCallback(() => {
    if (timerRef.current) {
      clearTimeout(timerRef.current);
      timerRef.current = null;
    }
  }, []);

  const pickUnseenIdle = useCallback((): string => {
    if (seenIdle.current.size >= FACES.idle.length) {
      seenIdle.current.clear();
    }
    const remaining = FACES.idle
      .map((f, i) => ({ f, i }))
      .filter(({ i }) => !seenIdle.current.has(i));
    const pick = randomFrom(remaining);
    seenIdle.current.add(pick.i);
    return pick.f;
  }, []);

  // On expand: show happy → idle
  useEffect(() => {
    if (isCollapsed) return;

    clearPending();
    clickCount.current = 0;
    seenIdle.current.clear();

    setCurrentFace(randomFrom(FACES.happy));
    timerRef.current = setTimeout(() => {
      setCurrentFace(pickUnseenIdle());
    }, HAPPY_DURATION_MS);

    return clearPending;
  }, [isCollapsed, clearPending, pickUnseenIdle]);

  const handleClick = useCallback(() => {
    clearPending();
    clickCount.current += 1;

    if (clickCount.current >= ANGRY_CLICK_THRESHOLD) {
      // Show angry, then reset
      setCurrentFace(randomFrom(FACES.angry));
      clickCount.current = 0;
      seenIdle.current.clear();
      timerRef.current = setTimeout(() => {
        setCurrentFace(pickUnseenIdle());
      }, ANGRY_DURATION_MS);
    } else {
      setCurrentFace(pickUnseenIdle());
    }
  }, [clearPending, pickUnseenIdle]);

  return (
    <img
      src={currentFace}
      alt="BMO face"
      onClick={handleClick}
      className="w-full h-full rounded-2xl cursor-pointer select-none"
      draggable={false}
      style={{ objectFit: "cover" }}
    />
  );
}
