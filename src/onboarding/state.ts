import { useReducer } from "react";

export interface HotkeyBinding {
  mode: "fn" | "ctrl+option+space";
}

export interface OnboardingState {
  step: 1 | 2 | 3 | 4;
  key: string;
  binding: HotkeyBinding | null;
}

type Action =
  | { type: "SET_STEP"; step: OnboardingState["step"] }
  | { type: "SET_KEY"; key: string }
  | { type: "SET_BINDING"; binding: HotkeyBinding | null };

const initialState: OnboardingState = {
  step: 1,
  key: "",
  binding: { mode: "fn" },
};

function reducer(state: OnboardingState, action: Action): OnboardingState {
  switch (action.type) {
    case "SET_STEP":
      return { ...state, step: action.step };
    case "SET_KEY":
      return { ...state, key: action.key };
    case "SET_BINDING":
      return { ...state, binding: action.binding };
    default:
      return state;
  }
}

export function useOnboardingState() {
  const [state, dispatch] = useReducer(reducer, initialState);
  return { state, dispatch };
}
