# JWT Validation Policy Example

This example showcases the helpers provided by the Poliy Development Kit JWT library to extract, parse and validate JWT tokens.

## Prerequisites

-- link to prerequisites doc page --

## Policy use case

A local library in a small town is using an open source software that manages the books inventory, and provides an API to keep track of the book consults and borrowings made by the library customers. The software contains an authentication system that leverages JSON Web tokens.

JSON Web tokens are an industry standard method to represent claims securely between different parties. These tokens can transport securely small sets of data and be trusted because they can be digitally signed, either using a secret (with the HMAC algorithm) or with a set of private and public keys (using RSA or ECDSA algorithms).

Now the library implemented a benefit system for regular customers that take care of the books and return them in a timely manner. Again, they are using a new open source software, but they need to reuse the authentication system provided by the book manager software.

With the signing keys used by the book management service to sign the JWT tokens, a policy can provide an authentication mechanism that reuses these tokens. This policy will be responsible for validating the signature of the tokens, ensuring they are not expired, and obtaining the role of the user (can be a customer or an administrator). All this information is contained in the tokens, the policy simply must make sure the token is current and trustworthy, and extract the required information to forward it to the benefits service.
