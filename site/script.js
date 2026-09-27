const observer = new IntersectionObserver(
  (entries) => {
    for (const entry of entries) {
      if (entry.isIntersecting) {
        entry.target.classList.add("visible");
        observer.unobserve(entry.target);
      }
    }
  },
  { threshold: 0.12 },
);

for (const element of document.querySelectorAll(".feature-card, .milestone, .pipeline-node, .architecture-note")) {
  element.classList.add("reveal");
  observer.observe(element);
}

const header = document.querySelector(".site-header");
window.addEventListener("scroll", () => {
  const scrolled = window.scrollY > 20;
  header?.classList.toggle("scrolled", scrolled);
});
