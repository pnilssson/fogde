const pattern =
  /(?:^|:)!?-?((?:[mp](?:[trblxyse]|[bi][se]?)?|space-[xy]|gap(?:-[xy])?))-(.+?)(?:!)?$/;

function breaks(token) {
  const match = pattern.exec(token);
  if (!match) {
    return false;
  }

  const [, property = "", value = ""] = match;
  if (property.startsWith("m") && value === "auto") {
    return false;
  }
  if (property.startsWith("space-") && value === "reverse") {
    return false;
  }
  if (/^[[(].*--/.test(value)) {
    return false;
  }
  if (!/^(?:[\d.]+|px|\[.*\]|\(.*\))$/.test(value)) {
    return false;
  }

  return !(value === "2" || (/^\d+$/.test(value) && Number(value) % 4 === 0));
}

export const spacing = {
  meta: {
    type: "suggestion",
    schema: [],
    messages: {
      token:
        'Use a Tailwind spacing number divisible by four, or 2, instead of "{{utility}}".',
    },
  },
  create(context) {
    function check(node, text) {
      if (typeof text !== "string") {
        return;
      }

      for (const token of text.split(/\s+/)) {
        if (breaks(token)) {
          context.report({
            node,
            messageId: "token",
            data: { utility: token },
          });
        }
      }
    }

    return {
      Literal: (node) => {
        check(node, node.value);
      },
      TemplateElement: (node) => {
        check(node, node.value.cooked);
      },
    };
  },
};
