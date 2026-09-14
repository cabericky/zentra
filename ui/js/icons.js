/**
 * Zentra SVG Icon Sprite Loader
 * Asynchronously loads modular SVG icon sprites into the DOM.
 */

export async function initIcons() {
  const container = document.getElementById('zen-icons-container');
  if (!container) return;

  try {
    const response = await fetch('assets/icons.svg');
    if (response.ok) {
      const svgText = await response.text();
      container.innerHTML = svgText;
    } else {
      console.warn('Failed to load assets/icons.svg, status:', response.status);
    }
  } catch (err) {
    console.error('Error loading SVG icons sprite:', err);
  }
}