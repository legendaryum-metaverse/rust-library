const config = {
  plugins:[
    require.resolve("prettier-plugin-sh"),
    require.resolve("prettier-plugin-sql"),

  ],
  language: 'postgresql',
  singleQuote: true
};

module.exports = config;
