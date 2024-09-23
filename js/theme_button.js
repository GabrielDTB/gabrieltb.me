// switch theme mode
if (document.getElementById('mode')) {
    document.getElementById('mode').addEventListener('click', () => {
    var timeout = .1;
    var style = document.createElement('style');
    style.innerHTML = `
        * {
        transition: background-color ${timeout}s ease-out,color ${timeout}s ease-out
        }
    `;
    document.head.appendChild(style);
    document.documentElement.classList.toggle('switch');
    localStorage.setItem('theme', document.documentElement.classList.contains('switch') ? 'switch' : 'default');
    setTimeout(() => document.head.removeChild(style), timeout * 1000);
  });
}
