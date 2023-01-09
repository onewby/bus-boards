/** @type {import('tailwindcss').Config} */
module.exports = {
  content: ['./src/**/*.{html,js,svelte,ts}'],
  theme: {
    extend: {
        backgroundImage: {
            // <a href="https://flickr.com/photos/126337928@N05/23714174510/in/photolist-C8xkTL-yuyTG4" target="_blank">"Park Place, Cathays, Cardiff"</a> by <a href="https://flickr.com/photos/126337928@N05/" target="_blank">Jeremy Segrott</a> is licensed under <a href="http://creativecommons.org/licenses/by/2.0" target="_blank">CC BY 2.0</a>
            cardiffBus: "url('https://live.staticflickr.com/5749/23714174510_a2c89bdcf3_b.jpg')"
        }
    },
  },
  plugins: [
      require('@tailwindcss/typography'),
      require('@tailwindcss/forms'),
  ],
}
