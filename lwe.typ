= Basic Problem Statement

LWE: $B = A dot S + E mod Q$

Ring LWE: $B(x) = A(x) * S(x) + E(x) mod x^Q-1$

Lattice LWE: $arrow(B) = arrow(A) dot arrow(S) + arrow(E)$

= Encryption and Decryption in LWE

Let $(C_0, C_1)=(A, B+M)$ be the ciphertext of a message $M$, $"Enc"(M, S)$.

$ "Dec"(C_0,C_1,S) = C_1-C_0 dot S $
$ = (B+M)-A dot S $
$ = (A dot S + E +M)-A dot S $
$ = A dot S -A dot S + E + M $
$ = E + M $
$ approx M $

= Operations On Ciphertexts
Let $(C_0, C_1)=(A,B+M)$ and $(C_0 ', C_1 ')=(A',B'+M')$


Let $(C_0, C_1)$ and $(C_0 ', C_1 ')$

Try component-wise addition:


$ "Dec"(C_0+C_0 ', C_1+C_1 ', S) = (C_1+C_1')-(C_0+C_0') dot S $
$ = (B+M+B'+M')-(A+A') dot S $
$ = (A dot S + E + M + A' dot S + E' + M')-(A+A') dot S $
$ = (A dot S - A dot S ) + (A' dot S - A' dot S) + E + M + E' + M' $
$ = (E + E') + (M + M') $

Try component-wise multiplication:




Addition: (E+E') < d
Multiplication: 
