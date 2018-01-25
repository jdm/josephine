\section{Appendix}

[Appendix goes here]

\begin{comment}

\begin{code}
data Bool : Set where
  true false : Bool
{-# BUILTIN BOOL Bool #-}
{-# BUILTIN TRUE true #-}
{-# BUILTIN FALSE false #-}

postulate /String/ : Set
{-# BUILTIN STRING  /String/ #-}

private
 primitive
  primStringAppend   : /String/ → /String/ → /String/
  primStringEquality : /String/ → /String/ → Bool

data /Nat/ : Set where
  zero : /Nat/
  1+  : /Nat/ -> /Nat/
{-# BUILTIN NATURAL /Nat/  #-}

data _/equals/_ {a} {A : Set a} (x : A) : A → Set a where
  refl : (x /equals/ x)
{-# BUILTIN EQUALITY _/equals/_  #-}
{-# BUILTIN REFL refl #-}

private
 primitive
  primTrustMe : ∀ {a} {A : Set a} {x y : A} → x /equals/ y

infixr 3 _+_
_+_ : /Nat/ → /Nat/ → /Nat/
zero + y = y
1+ x + y = 1+ (x + y)

_⊔_ : /Nat/ → /Nat/ → /Nat/
zero ⊔ y = y
x ⊔ zero = x
1+ x ⊔ 1+ y = 1+ (x ⊔ y)

data ⊥ : Set where

_/neq/_ : ∀ {A : Set} → A → A → Set
(x /neq/ y) = (x /equals/ y) → ⊥

infixr 5 _,_
record Σ (A : Set) (B : A → Set) : Set where
  constructor _,_
  field /fst/ : A
  field /snd/ : B /fst/
syntax Σ A (λ x → M) = /exists/ x /in/ A /st/ M

∃ : {A : Set} → (A → Set) → Set
∃ {A} B = Σ A B
syntax ∃ (λ x → M) = /exists/ x /st/ M
infixr 10 ∃

∄ : {A : Set} → (A → Set) → Set
∄ P = (∃ P) → ⊥
syntax ∄ (λ x → M) = /nexists/ x /st/ M
infixr 10 ∄

infixr 5 _/times/_
_/times/_ : Set → Set → Set
(A /times/ B) = Σ A (λ _ → B)

infixr 9 _/andalso/_
_/andalso/_ = _/times/_

_/fun/_ : Set → Set → Set
(A /fun/ B) = A → B

infixr 8 _/implies/_
_/implies/_ : Set → Set → Set
(A /implies/ B) = A → B

infixl 7 _/where/_
_/where/_ : Set → Set → Set
(A /where/ B) = B → A

infixl 7 _/wherenot/_
_/wherenot/_ : Set → Set → Set
(A /wherenot/ B) = (A /where/ (B → ⊥))

data /Lift/ (A : Set) : Set where
  /bot/ : /Lift/ A
  /lift/ : A → /Lift/ A

/emptyset/ : ∀ {A B : Set} → A → /Lift/(B) 
/emptyset/(x) = /bot/

_/override/_ : ∀ {A B : Set} → (A → /Lift/(B)) → (A → /Lift/(B)) → (A → /Lift/(B))
(f /override/ g)(x) with f(x)
... | /bot/ = g(x)
... | /lift/(y) = /lift/(y)

_=S?_ : (x y : /String/) → /Lift/(x /equals/ y)
x =S? y with primStringEquality x y
... | true = /lift/ primTrustMe
... | false = /bot/

_=ℕ?_ : (x y : /Nat/) → /Lift/(x /equals/ y)
zero =ℕ? zero = /lift/ refl
zero =ℕ? 1+ y = /bot/
1+ x =ℕ? zero = /bot/
1+ x =ℕ? 1+ y  with x =ℕ? y
1+ x =ℕ? 1+ .x | /lift/(refl) = /lift/(refl) 
1+ x =ℕ? 1+ y  | /bot/ = /bot/

data /List/ (A : Set) : Set where
  /unit/ : /List/(A)
  _,_ : A → /List/(A) → /List/(A)

/singleton/ : ∀ {A} → A → /List/(A)
/singleton/ x = (x , /unit/)

/length/ : ∀ {A} → /List/(A) → /Nat/
/length/ /unit/ = 0
/length/ (x , xs) = 1 + /length/(xs)

data _/member/_ {A : Set} (x : A) : /List/(A) → Set where
  here : ∀ {xs} → (x /member/ (x , xs))
  there : ∀ {y xs} → (x /member/ xs) → (x /member/ (y , xs))

max : /List/(/Nat/) → (/Nat/ → /Nat/) → /Nat/
max /unit/ f = 0
max (x , xs) f = f(x) ⊔ max xs f
syntax max cs (λ c → N) = /max/ c /in/ cs /st/ N

infixr 3 _/append/_
_/append/_ : ∀ {A} → /List/(A) → /List/(A) → /List/(A)
/unit/ /append/ ys = ys
(x , xs) /append/ ys = (x , (xs /append/ ys))
\end{code}

\begin{code}
/Word/ = /Nat/
/Var/ = /String/

data /Slice/ : Set where
  /slice/_[_/dots/_] : /Word/ → /Word/ → /Word/ → /Slice/

data /Type/ : Set where
  /enum/_/st/_ : /List/(/Word/) → (/Word/ → /Type/) → /Type/
  /unit/ : /Type/

data /Pat/ : Set where
  /varref/_ : /Var/ → /Pat/
  /var/_/in/_ : /Var/ → /Type/ → /Pat/
  /addr/_ : /Pat/ → /Pat/
  /tagged/_/with/ : /Type/ → /Word/ → /Pat/ → /Pat/
  /unit/ : /Pat/

data /Exp/ : Set where
  /var/_ : /Var/ → /Exp/
  /val/_ : /List/(/Word/) → /Exp/
  /slice/_[_/dots/_] : /Word/ →  /Word/ →  /Word/ → /Exp/
  /tagged/_/with/ : /Type/ → /Word/ → /Exp/ → /Exp/
  /iflet/_/equals/_/thn/_/els/_/telfi/ : /Pat/ → /Exp/ → /Exp/ → /Exp/ → /Exp/
  /iflet/_/equals//ast/_/thn/_/els/_/telfi/ : /Pat/ → /Exp/ → /Exp/ → /Exp/ → /Exp/
  /unit/ : /Exp/
\end{code}

\begin{code}
_\\ : ∀ {ℓ} {A : Set(ℓ)} → A → A
x \\ = x
\end{code}

\end{comment}

\begin{code}
/Memory/ = (/Word/ /fun/ /Lift/(/Word/))
\end{code}

\begin{code}
/sizeof/ : /Type/ /fun/ /Nat/ \\
/sizeof/ (/enum/ /vec/c /st/ T) = 1 + (/max/ c /in/ /vec/c /st/ /sizeof/(T(c))) \\
/sizeof/ /unit/ = 0
\end{code}

\begin{code}
/Value/ = /List/(/Word/) \\
\end{code}

\begin{comment}
\begin{code}
_/comma/_ : ∀ {A} → /Lift/(A) → /Lift/(/List/(A)) → /Lift/(/List/(A))
/bot/ /comma/ _ = /bot/
/lift/ x /comma/ /bot/ = /bot/
/lift/ x /comma/ /lift/ xs = /lift/ (x , xs)

/sem//slice/_[_/dots/_]/mes/ : /Word/ → /Word/ → /Word/ → /Memory/ → /Lift/(/Value/)
\end{code}
\end{comment}

\begin{code}
/sem//slice/ p [ x /dots/ 0 ]/mes/ /rho/ = /lift/ /unit/ \\
/sem//slice/ p [ 0 /dots/ (1+ y) ]/mes/ /rho/ = (/rho/(p) /comma/ /sem//slice/ (1+ p) [ 0 /dots/ y ]/mes/ /rho/) \\
/sem//slice/ p [ (1+ x) /dots/ (1+ y) ]/mes/ /rho/ = /sem//slice/ (1+ p) [ x /dots/ y ]/mes/ /rho/
\end{code}

\begin{comment}
\begin{code}
_/oplus/_/mapsto/_ : /Memory/ → /Word/ → /Value/ → /Lift/(/Memory/)
(ρ /oplus/ p /mapsto/ /unit/) = /lift/(ρ)
(ρ /oplus/ p /mapsto/ (v , V)) with ρ(p) | (ρ /oplus/ (1+ p) /mapsto/ V)
... | /bot/ | /lift/(ρ′) = /lift/ ρ″ where
  ρ″ : /Memory/
  ρ″(q)  with p =ℕ? q
  ρ″(.p) | /lift/(refl) = /lift/(v)
  ρ″(q)  | /bot/ = ρ′(q)
... | _     | _          = /bot/

_[_:=_] : /Exp/ → /Var/ → /Value/ → /Exp/
_[_:=_] = {!!}

\end{code}
\end{comment}

\begin{comment}
\begin{code}
data _/step/_ : (/Memory/ /times/ /Exp/) → (/Memory/ /times/ /Exp/) → Set where
\end{code}
\end{comment}

\begin{code}
  %iflet*-addr-var : ∀ /rho/ x T p V M N n →
    (/rho/ , /iflet/ /var/ x /in/ T /equals//ast/ /val/ /singleton/ p /thn/ M /els/ N /telfi/) /step/ (/rho/ , M [ x := V ]) /where/
    /sem//slice/ p [ 0 /dots/ n ]/mes/ /rho/ /equals/ /lift/ V /andalso/
    n /equals/ /sizeof/(T) \\
  %iflet*-ref : ∀ /rho/ x M N p → 
    (/rho/ , /iflet/ /varref/ x /equals//ast/ /val/ /singleton/ p /thn/ M /els/ N /telfi/) /step/ (/rho/ , M [ x := /singleton/ p ]) \\
  %iflet*-tagged : ∀ /rho/ T c X M N p → 
    (/rho/ , /iflet/ /tagged/ T /with/ c(X) /equals//ast/ /val/ /singleton/ p /thn/ M /els/ N /telfi/) /step/ (/rho/ , /iflet/ X /equals//ast/ /val/ /singleton/ (1+ p) /thn/ M /els/ N /telfi/) \\
  %iflet*-tagged-fail : ∀ /rho/ T c X M N p → 
    (/rho/ , /iflet/ /tagged/ T /with/ c(X) /equals//ast/ /val/ /singleton/ p /thn/ M /els/ N /telfi/) /step/ (/rho/ , N)  /where/
    /lift/ c /neq/ /rho/(p) \\
  %iflet*-addr-unit : ∀ /rho/ p M N →
    (/rho/ , /iflet/ /unit/ /equals//ast/ /val/ /singleton/ p /thn/ M /els/ N /telfi/) /step/ (/rho/ , M) \\
  %iflet*-addr : ∀ /rho/ X M N p q → 
    (/rho/ , /iflet/ /addr/ X /equals//ast/ /val/ /singleton/ p /thn/ M /els/ N /telfi/) /step/ (/rho/ , /iflet/ X /equals//ast/ /val/ /singleton/ q /thn/ M /els/ N /telfi/) /where/
    /lift/ q /equals/ /rho/(p)
\end{code}


\begin{code}
  %iflet-alloc : ∀ /rho/ X V M N /rhoP/ p →
    (/rho/ , /iflet/ X /equals/ /val/ V /thn/ M /els/ N /telfi/) /step/ (/rhoP/ , /iflet/ X /equals//ast/ /val/ /singleton/ p /thn/ M /els/ N /telfi/) /where/
    /lift/ /rhoP/ /equals/ (/rho/ /oplus/ p /mapsto/ V)
\end{code}

\begin{code}
  %iflet-redn : ∀ /rho/ X L M N /rhoP/ /LP/ →
    (/rho/ , /iflet/ X /equals/ L /thn/ M /els/ N /telfi/) /step/ (/rhoP/ , /iflet/ X /equals/ /LP/ /thn/ M /els/ N /telfi/) /where/
    (/rho/ , L) /step/ (/rhoP/ , /LP/) \\
  %iflet*-redn : ∀ /rho/ X L M N /rhoP/ /LP/ →
    (/rho/ , /iflet/ X /equals//ast/ L /thn/ M /els/ N /telfi/) /step/ (/rhoP/ , /iflet/ X /equals//ast/ /LP/ /thn/ M /els/ N /telfi/) /where/
    (/rho/ , L) /step/ (/rhoP/ , /LP/) \\
\end{code}

\begin{comment}
\begin{code}
/Subst/ = /Var/ → /Lift/ /Slice/
_/mapsto/_ : /Var/ → /Slice/ → /Subst/
(x /mapsto/ S) y  with x =S? y
(x /mapsto/ S) y  | /bot/ = /bot/
(x /mapsto/ S) .x | /lift/ refl = /lift/ S
\end{code}
\end{comment}

\begin{comment}
\begin{code}
data _/bigSubst/_ : (/Memory/ /times/ /Pat/ /times/ /Value/) → (/Memory/ /times/ /Subst/) → Set where
\end{code}
\end{comment}

\begin{code}
  %var : ∀ /rho/ x T V p n →
    (/rho/ , /var/ x /in/ T , V) /bigSubst/ (/rho/ , (x /mapsto/ /slice/ p [ 0 /dots/ n ])) /where/
    n /equals/ /sizeof/(T) \\
  %tagged : ∀ /sigma/ /rho/ /rhoP/ X T c V W /vec/c U →
    (/rho/ , X , V) /bigSubst/ (/rhoP/ , /sigma/) /implies/
    (/rho/ , /tagged/ T /with/ c(X) , (c , V /append/ W)) /bigSubst/ (/rhoP/ , /sigma/) /where/
    T /equals/ /enum/ /vec/c /st/ U /andalso/
    /length/(V) /equals/ /sizeof/(U(c))
\end{code}

\begin{comment}
\begin{code}
data _/bigStep/_ : (/Subst/ /times/ /Memory/ /times/ /Exp/) → (/Memory/ /times/ /Value/) → Set where
\end{code}
\end{comment}

\begin{code}
  %var : ∀  /sigma/ /rho/ x V p i j →
    (/sigma/ , /rho/ , /var/ x) /bigStep/ (/rho/ , V) /where/
    /sigma/(x) /equals/ /lift/ /slice/ p [ i /dots/ j ]  /andalso/
    /sem//slice/ p [ i /dots/ j ]/mes/ /rho/ /equals/ /lift/ V \\
  %tagged : ∀ /sigma/ /rho/ /rhoP/ T c M V W →
    (/sigma/ , /rho/ , M) /bigStep/ (/rhoP/ , V) /implies/
    (/sigma/ , /rho/ , /tagged/ T /with/ c(M)) /bigStep/ (/rhoP/ , (c , V /append/ W)) /where/
    /length/(c , V /append/ W) /equals/ /sizeof/(T) \\
  %ifletT : ∀ /sigma/ /rho/ X L M N /sigmaP/ /rhoP/ /rhoPP/ /rhoPPP/ V W →
    (/sigma/ , /rho/ , L) /bigStep/ (/rhoP/ , V) /andalso/
    ((/sigma/ /override/ /sigmaP/) , /rhoPP/ , M) /bigStep/ (/rhoPPP/ , W) /implies/
    (/sigma/ , /rho/ , /iflet/ X /equals/ L /thn/ M /els/ N /telfi/) /bigStep/ (/rhoPPP/ , W) /where/
    (/rhoP/ , X , V) /bigSubst/ (/rhoPP/ , /sigmaP/) \\
  %ifletF : ∀ /sigma/ /rho/ X L M N /rhoP/ /rhoPP/ V W →
    (/sigma/ , /rho/ , L) /bigStep/ (/rhoP/ , V) /andalso/
    (/sigma/ , /rhoP/ , N) /bigStep/ (/rhoPP/ , W) /implies/
    (/sigma/ , /rho/ , /iflet/ X /equals/ L /thn/ M /els/ N /telfi/) /bigStep/ (/rhoPP/ , W)
\end{code}

\begin{code}
/SType/ = (/Var/ /fun/ /Lift/(/Type/)) \\
/MType/ = (/Word/ /fun/ /Lift/(/Type/)) \\
\end{code}

\begin{comment}
\begin{code}
data _/dashv/_/in/_ : /SType/ → /Pat/ → /Type/ → Set where
\end{code}
\end{comment}

\begin{comment}
\begin{code}
data _/vdash/_/in/_ : /SType/ → /Exp/ → /Type/ → Set where
\end{code}
\end{comment}

\begin{code}
  %tagged : ∀ /Gamma/ T c M /vec/c U →
    /Gamma/ /vdash/ M /in/ U(c) /implies/
    /Gamma/ /vdash/ /tagged/ T /with/ c(M) /in/ T /where/
    T /equals/ (/enum/ /vec/c /st/ U) /andalso/
    c /member/ /vec/c \\
  %iflet : ∀ /Gamma/ /GammaP/ X L M N T U →
    /Gamma/ /vdash/ L /in/ U /andalso/
    (/Gamma/ /override/ /GammaP/) /vdash/ M /in/ T /andalso/
    /Gamma/ /vdash/ N /in/ T /implies/
    /Gamma/ /vdash/ /iflet/ X /equals/ L /thn/ M /els/ N /telfi/ /in/ T /where/
    /GammaP/ /dashv/ X /in/ U
\end{code}

\begin{comment}
\begin{code}
data _/vdashv/_/in/_ : /MType/ → /Value/ → /Type/ → Set where
\end{code}
\end{comment}

\begin{code}
  %tagged : ∀ /Delta/ T c V W /vec/c U →
    /Delta/ /vdashv/ V /in/ U(c) /implies/
    /Delta/ /vdashv/ (c , V /append/ W) /in/ T /where/
    T /equals/ (/enum/ /vec/c /st/ U) /andalso/
    c /member/ /vec/c \\
\end{code}

\begin{code}
%safety : ∀ /Gamma/ (/Delta/ : /MType/) {M} {T} /sigma/ /rho/ /rhoP/ V →
  (/Gamma/ /vdash/ M /in/ T) /andalso/
  (/sigma/ , /rho/ , M) /bigStep/ (/rhoP/ , V) /implies/
  (/exists/ /DeltaP/ /st/ (/DeltaP/ /vdashv/ V /in/ T))
\end{code}

\begin{comment}
\begin{code}
%safety Γ Δ σ ρ ρ′ ._ (%tagged .Γ .(/enum/ cs /st/ U) c M cs U (refl , c∈cs) M∈Uc , %tagged .σ .ρ .ρ′ .(/enum/ cs /st/ U) .c .M V W _ M⇓V)
  with %safety Γ Δ σ ρ ρ′ V (M∈Uc , M⇓V)
... | (Δ′ , V∈T)
  = (Δ′ , %tagged Δ′ (/enum/ cs /st/ U) c V W cs U (refl , c∈cs) V∈T)
%safety Γ Δ σ ρ ρ‴ V (%iflet .Γ Γ′ X L M N T U X∈U (L∈U , M∈T , N∈T) , %ifletT .σ .ρ .X .L .M .N σ′ ρ′ ρ″ .ρ‴ W .V x (L⇓W , M⇓V))
  = %safety (Γ /override/ Γ′) Δ (σ /override/ σ′) ρ″ ρ‴ V (M∈T , M⇓V)
%safety Γ Δ σ ρ ρ″ V (%iflet .Γ Γ′ X L M N T U X∈U (L∈U , M∈T , N∈T) , %ifletF .σ .ρ .X .L .M .N ρ′ .ρ″ W .V (L⇓W , N⇓V))
  = %safety Γ Δ σ ρ′ ρ″ V (N∈T , N⇓V)
\end{code}
\end{comment}