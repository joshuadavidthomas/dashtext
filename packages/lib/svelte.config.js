/** @type {import('@sveltejs/kit').Config} */
const config = {
  kit: {
    alias: {
      "$lib": "./src",
      "$lib/*": "./src/*"
    }
  }
};

export default config;
