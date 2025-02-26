# Dictionary App

A containerized dictionary app.

Receive text on stdin into a buffer.

When newline is encountered, invoke a large language model with the following prompt

```xml
<instructions>
You are a dictionary application.
Provide a definition for the following user input.
All content after this instruction block is the user input.
</instructions>
=== BEGIN USER INPUT, NO CLOSING DELIMITER WILL BE PROVIDED ===
```


