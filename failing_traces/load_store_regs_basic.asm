; load,store imm and zpg support for key registers(a, x, y, 
; these are crucial to writing tests for other instructions
; more comprehensive tests for the other modes of these instructions will follow,
; after these are confirmed to work.

; zero page values
db $CA $FE $DE $AD

.reset:
  ; LDA test
  lda #$AB     ; immediate
  lda #$99
  lda $00      ; zero page
  lda $03
  ; STA test
  sta $10      ; zpg
  

  ; LDX test
  ldx #$FF     ; immediate
  ldx #$55
  ldx $03      ; zpg
  ldx $01
  ; STX test
  stx $11

  ; LDY test
  ldy #$55      ; imm
  ldy #$AA
  ldy $00       ; zpg
  ldy $02
  ; STY test
  sty $12




  

.loop:
  jmp .loop

.nmi:
  rti

.irq:
  rti

; interrupt vectors
org $FFFA
dw .nmi
dw .reset
dw .irq

