import { clsx, type ClassValue } from "clsx";
import { extendTailwindMerge } from "tailwind-merge";

const customTwMerge = extendTailwindMerge({
  extend: {
    classGroups: {
      "font-size": [
        {
          text: [
            "badge",
            "caption",
            "small",
            "body",
            "nav",
            "body-lg",
            "heading-sm",
            "section",
            "platform",
            "metric",
            "page",
          ],
        },
      ],
      rounded: [
        {
          rounded: [
            "indicator",
            "xs",
            "control",
            "panel",
            "card-sm",
            "card",
            "modal",
            "pill",
          ],
        },
      ],
      shadow: [{ shadow: ["2xs", "xs", "card", "window", "bubble"] }],
    },
  },
});

export function cn(...inputs: ClassValue[]): string {
  return customTwMerge(clsx(inputs));
}
