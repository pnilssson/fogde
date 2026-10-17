export function followSystemTheme() {
  const dark = window.matchMedia("(prefers-color-scheme: dark)");
  const apply = () => {
    document.documentElement.classList.toggle("dark", dark.matches);
  };

  apply();
  dark.addEventListener("change", apply);
}
