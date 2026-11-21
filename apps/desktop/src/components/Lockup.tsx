import { motion } from "motion/react";
import { useTranslation } from "react-i18next";
import foldRibbon from "../assets/fold-ribbon.svg";
import { useAppStore } from "../store";

// The intro plays once per session, not every time the start screen comes back.
let introPlayed = false;

/**
 * The product name on the start screen: the Fold ribbon (".aep" with its arrow) above
 * "Downgrader", the arrow running down past the end of the word to the depth of the "g".
 * Proportions from the owner's layout: ribbon height 1.96x the font size, its left edge 3.27x the
 * font size from the D, its top 1.54x the font size above the capitals (4 px higher than measured, by eye).
 */
export function Lockup() {
  const { t } = useTranslation();
  // The ribbon reaches 27 px above the word, into the title bar; under the update banner it moves down instead.
  const banner = useAppStore((s) => s.update.status !== "none" && !s.update.bannerDismissed);
  const play = !introPlayed;
  introPlayed = true;
  return (
    // In a short window everything scales by 0.8.
    <h1
      className={`relative shrink-0 pt-[46px] [@media(max-height:620px)]:pt-[37px] ${banner ? "mt-5" : ""}`}
      aria-label={t("app.name")}
    >
      {/* The shadow is on the wrapper: the clip that reveals the ribbon would cut a shadow on the image itself. */}
      <span className="pointer-events-none absolute top-[-31px] left-[180px] drop-shadow-[0_10px_12px_var(--lockup-shadow)] [@media(max-height:620px)]:top-[-25px] [@media(max-height:620px)]:left-[144px]">
        <motion.img
          src={foldRibbon}
          alt=""
          initial={play ? { clipPath: "inset(0 100% 0 0)", x: -14, y: 6 } : false}
          animate={{ clipPath: "inset(0 0% 0 0)", x: 0, y: 0 }}
          transition={{ duration: 0.62, ease: [0.22, 1, 0.36, 1] }}
          className="block h-[108px] w-auto [@media(max-height:620px)]:h-[86px]"
        />
      </span>
      <motion.span
        aria-hidden
        initial={play ? { opacity: 0, x: -18 } : false}
        animate={{ opacity: 1, x: 0 }}
        transition={{ duration: 0.52, delay: 0.3, ease: [0.22, 1, 0.36, 1] }}
        className="relative block text-[55px] leading-none font-extrabold tracking-display [@media(max-height:620px)]:text-[44px]"
      >
        Downgrader
      </motion.span>
    </h1>
  );
}
