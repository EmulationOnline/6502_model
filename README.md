# 6502 Model

This is a cycle accurate model of the 65C02 chip. This model is
produced with the support of the ChipLab, which provides cycle-by-cycle
traces of all external busses while executing programs on a real chip.

## Contributing
Contributions welcome! If you would like to improve the model, a good workflow is
1. Find something that isn't working. See the roadmap or join our Discord.
2. Write a 6502 program that demonstrates desired behavior
3. Run the program on the lab, and collect the signed trace.
4. Add the trace to this repo as a test case, which should fail.
5. Implement the desired functionality.

## Roadmap / Currently implemented
The list below gives an idea of what is currently supported. 
Unchecked boxes are planned but not yet complete.
- [ ] All official instructions are implemented (sans flags)
- [ ] Flags are added to official instructions
- [ ] NMI interrupt is implemented
- [ ] IRQ interrupt is implemented
- [ ] Unofficial instructions are implemented, + flags

## Discord
We coordinate development discussion on Discord.
https://discord.gg/uwx87FAYMu

