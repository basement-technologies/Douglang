hljs.registerLanguage('douglang', function(hljs) {
  return {
    name: 'Douglang',
    contains: [
      { className: 'keyword', begin: /\+set|-set|\*set|\/set|%set|set|tts|prediction|Believers|Doubters|win|loop|guoD|Rigged/ },
      { className: 'keyword', begin: /(Doug)+/ },
      { className: 'keyword', begin: /Bald/ },
      { className: 'number', begin: /(\d+(\.\d+)?|\.\d+)/, relevance: 0 },
      hljs.QUOTE_STRING_MODE,
      { className: 'comment', begin: /D:/, end: /:D/ },
    ]
  };
});

// Highlight inline code blocks
document.addEventListener("DOMContentLoaded", () => {
  document.querySelectorAll('code.language-douglang').forEach((el) => hljs.highlightElement(el));
});