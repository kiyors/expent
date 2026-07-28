import { GooeyToaster, gooeyToast } from "goey-toast";
import { useTheme } from "next-themes";

import "goey-toast/styles.css";

const Toaster = ({ ...props }: React.ComponentProps<typeof GooeyToaster>) => {
  const { theme = "system" } = useTheme();

  return (
    <GooeyToaster
      theme={theme as "light" | "dark"}
      toastOptions={{
        classNames: {
          toast: "cn-toast",
        },
      }}
      {...props}
    />
  );
};

const toast = gooeyToast;

export { Toaster, toast };
