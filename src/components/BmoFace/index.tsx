import { useCallback, useEffect, useRef, useState } from "react";
import { useBmoStore } from "../../store";
import { DEFAULT_IDLE, FACES } from "./faces";

function randomFrom<T>(arr: readonly T[]): T {
  return arr[Math.floor(Math.random() * arr.length)];
}

const ANGRY_CLICK_THRESHOLD = 5;
const HAPPY_DURATION_MS = 2000;
const ANGRY_COOLDOWN_MS = 3000;
const IDLE_RETURN_MS = 5000;

export function BmoFace() {
  const isCollapsed = useBmoStore((s) => s.isCollapsed);

  const [currentFace, setCurrentFace] = useState<string>(randomFrom(FACES.idle));
  const [disabled, setDisabled] = useState(false);
  const seenIdle = useRef<Set<number>>(new Set());
  const clickCount = useRef(0);
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const idleReturnRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const clearPending = useCallback(() => {
    if (timerRef.current) {
      clearTimeout(timerRef.current);
      timerRef.current = null;
    }
  }, []);

  const clearIdleReturn = useCallback(() => {
    if (idleReturnRef.current) {
      clearTimeout(idleReturnRef.current);
      idleReturnRef.current = null;
    }
  }, []);

  const scheduleIdleReturn = useCallback(() => {
    clearIdleReturn();
    idleReturnRef.current = setTimeout(() => {
      setCurrentFace(DEFAULT_IDLE);
      clickCount.current = 0;
      seenIdle.current.clear();
    }, IDLE_RETURN_MS);
  }, [clearIdleReturn]);

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
    clearIdleReturn();
    clickCount.current = 0;
    seenIdle.current.clear();
    setDisabled(false);

    setCurrentFace(randomFrom(FACES.happy));
    timerRef.current = setTimeout(() => {
      setCurrentFace(DEFAULT_IDLE);
    }, HAPPY_DURATION_MS);

    return () => {
      clearPending();
      clearIdleReturn();
    };
  }, [isCollapsed, clearPending, clearIdleReturn, pickUnseenIdle, scheduleIdleReturn]);

  const handleClick = useCallback(() => {
    if (disabled) return;

    clearPending();
    clearIdleReturn();
    clickCount.current += 1;

    if (clickCount.current >= ANGRY_CLICK_THRESHOLD) {
      // Show angry face and disable clicks for cooldown
      setCurrentFace(randomFrom(FACES.angry));
      setDisabled(true);
      clickCount.current = 0;
      seenIdle.current.clear();
      timerRef.current = setTimeout(() => {
        setDisabled(false);
        setCurrentFace(DEFAULT_IDLE);
      }, ANGRY_COOLDOWN_MS);
    } else {
      setCurrentFace(pickUnseenIdle());
      // After clicking to a non-default face, return to idle1 after inactivity
      scheduleIdleReturn();
    }
  }, [disabled, clearPending, clearIdleReturn, pickUnseenIdle, scheduleIdleReturn]);

  return (
    <img
      src={currentFace}
      alt="BMO face"
      onClick={handleClick}
      className={`w-full h-full rounded-2xl select-none ${disabled ? "cursor-default" : "cursor-pointer"}`}
      draggable={false}
      style={{ objectFit: "cover" }}
    />
  );
}
