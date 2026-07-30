(() => {
  const navToggle = document.querySelector(".nav-toggle");
  const navigation = document.querySelector(".site-navigation");

  if (navToggle && navigation) {
    navToggle.addEventListener("click", () => {
      const expanded = navToggle.getAttribute("aria-expanded") === "true";
      navToggle.setAttribute("aria-expanded", String(!expanded));
      navigation.dataset.open = String(!expanded);
    });
  }

  document.querySelectorAll("[data-tokimu-island]").forEach((island) => {
    const state = island.getAttribute("data-state") || "idle";
    island.setAttribute("data-state", state);
  });
})();
