// Loads data from one Rust command when a screen opens, and again whenever a
// value in `deps` changes (for example the selected crew member).
// Returns { data, error }; data is null while loading or after an error.
import { useEffect, useState, type DependencyList } from "react";

interface CommandState<T> {
  data: T | null;
  error: string | null;
}

export function useCommand<T>(load: () => Promise<T>, deps: DependencyList = []): CommandState<T> {
  const [state, setState] = useState<CommandState<T>>({ data: null, error: null });

  useEffect(() => {
    let isActive = true; // ignore a late answer if the screen was closed meanwhile
    load()
      .then((data) => isActive && setState({ data, error: null }))
      .catch((e: unknown) => isActive && setState({ data: null, error: String(e) }));
    return () => {
      isActive = false;
    };
    // `load` is recreated on every render; the caller lists what it depends on.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, deps);

  return state;
}
