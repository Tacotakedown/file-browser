import { useEffect, useState } from "react";
import { Button } from "@/components/ui/button";

const getStoredTheme = () => localStorage.getItem("theme");
const prefersDarkMode = window.matchMedia(
  "(prefers-color-scheme: dark)"
).matches;

const getInitialTheme = () => {
  return getStoredTheme() ?? (prefersDarkMode ? "dark" : "light");
};

export const ThemeToggle = () => {
  const [theme, setTheme] = useState(getInitialTheme());

  useEffect(() => {
    document.documentElement.classList.toggle("dark", theme === "dark");
    localStorage.setItem("theme", theme);
  }, [theme]);

  const toggleTheme = () => {
    setTheme((prevTheme) => (prevTheme === "dark" ? "light" : "dark"));
  };

  return (
    <Button onClick={toggleTheme} className="p-2 rounded ">
      {theme === "dark" ? "🌙" : "☀️"}
    </Button>
  );
};
