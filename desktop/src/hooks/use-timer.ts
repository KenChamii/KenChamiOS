import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { listen } from "@tauri-apps/api/event";
import { useEffect } from "react";
import {
  TIMER_EVENT,
  getTimer,
  getTimerSettings,
  pauseTimer,
  resumeTimer,
  setTimerSettings,
  startTimer,
  stopTimer,
  type TimerView,
} from "@/lib/api/timer";

const TIMER_KEY = ["timer"];

/** The running timer, kept fresh by the backend's per-second events. */
export function useTimer() {
  const queryClient = useQueryClient();
  const query = useQuery({ queryKey: TIMER_KEY, queryFn: getTimer });

  useEffect(() => {
    const unlisten = listen<TimerView | null>(TIMER_EVENT, (event) => {
      queryClient.setQueryData(TIMER_KEY, event.payload);
      // a finished focus phase adds a time entry
      void queryClient.invalidateQueries({ queryKey: ["time-entries"] });
    });
    return () => {
      void unlisten.then((stop) => stop());
    };
  }, [queryClient]);

  return query;
}

function useTimerMutation<T>(action: (arg: T) => Promise<TimerView | null | void>) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: action,
    onSuccess: (view) => queryClient.setQueryData(TIMER_KEY, view ?? null),
  });
}

export function useStartTimer() {
  return useTimerMutation(startTimer);
}

export function usePauseTimer() {
  return useTimerMutation(pauseTimer);
}

export function useResumeTimer() {
  return useTimerMutation(resumeTimer);
}

export function useStopTimer() {
  return useTimerMutation(stopTimer);
}

export function useTimerSettings() {
  return useQuery({ queryKey: ["timer-settings"], queryFn: getTimerSettings });
}

export function useSetTimerSettings() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: setTimerSettings,
    onSuccess: (settings) => queryClient.setQueryData(["timer-settings"], settings),
  });
}
