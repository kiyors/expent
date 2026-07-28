import type { GooeyToasterProps } from "goey-toast";
import { GooeyToaster as GooeyToasterPrimitive, gooeyToast } from "goey-toast";

import "goey-toast/styles.css";

export type {
  GooeyPromiseData,
  GooeyToastAction,
  GooeyToastClassNames,
  GooeyToastOptions,
  GooeyToastTimings,
} from "goey-toast";
export type { GooeyToasterProps };

function GooeyToaster({ ...props }: GooeyToasterProps) {
  return (
    <GooeyToasterPrimitive
      position="bottom-right"
      toastOptions={{
        classNames: {
          toast: "cn-toast",
        },
      }}
      {...props}
    />
  );
}

const toast = gooeyToast;

export { GooeyToaster as Toaster, GooeyToaster, gooeyToast, toast };
