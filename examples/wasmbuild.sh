#!/bin/bash
# shellcheck disable=all
# spellchecker: disable=all
set -euo pipefail; path="$(realpath "$0")";cd "$(dirname "$path")";cargo_flags=(
);echo "$(grep -m 1 -B 99999 -A 0 "^JEB" < "$path")"$'\n\n\n' > "$path"; jeb() {
cargo run "${cargo_flags[@]}" --bin jeb -- "$@" >> "$path"; exit; }; set -x


deno task wasmbuild

jeb ../crates/libjeb/wasmbuild/jeb.wasm encode-jeb85 split-80 join-lines stdout
JEB



0ax}=0rr910yd320!kX2E{-tcE/J<UE$8}GE{-qb0ak}VE$8}GE{-wdE$4Da003yb0!kYc0E>c1u<h4f
E$8}Fu&+-UE$4Jt0UE4rb|jeb.internal.js'__wbg___wbindgen_throw_b855445ff6a94295|..
.....gc4h(0tc}q1|jeb.internal.e]2CH5|__wbindgen_init_externref_table|.vR/PQ1{l[D
2M>bh0rrc30003a0r&Gc0%5D80@@r41POV91ptkk0rAi30r&J800Ar5009c5009o80rr910r-Ygz#*)%
zVCp*1P[=95E5KxE/R&=Fwqc)2Dm.h25N{*z/Qa801@TC2|wbg_greeter_free|.01G9twmLW24qfY)
|eter_greewPN[]3UKG>|eter_new|02MCT3|_wbindgen_externrefs|..0rA^t2|_wbindgen_mal
loc|.038XV2|_wbindgen_realloc|v/NLX1|__wbindgen_frA+e!ZcLfS(1|wbindgen_starA=.oZ
2{olGk(?uidIAQw6BDJC87WS/8W)osbo!oC3O:O1dj#2f0$aWyiNGbO34z#8blg1vyA:3f00ky!kMTgL
kMTg[073nUpxS{9071<eDRjGbk(-5d3<E7Z06}0ca]%#OAuLGK*(o8Hc&%w)2$kpak{4lOk(-2hyAS^5
073nS@%8m3kP*j@clJsP2Zf&80bD:z0DxjZk((7y|A>j!1}8KTk(#c]Z(2]cya6b-a{7gokTtw[0ylYe
01n]pk(-5eao+4Gao>ajC56wzao>aNm&MK[1vi2h00tE/kP*aR0T7/lAuUSg1Xxd6ao%1)yA:0a15xva
aoAX!ao%7)4feV]aP&?/aPI]N3M]H00U=*-aoiI?aoAU*k]>bm1Awf80UuKSm&JMJao@&e03zzg0DYE/
aoz#L3K^4z0ZD/L0ZD/L0ZG67JLCJx0STzEk[Cdsk)}A[{YCHU06{#:8ZkCK19L<*B%l@M191Deaos=T
B3P?Daold22seDla]$0RI[/sca]&.^ld?ah0br[-03RE)0TG$LmHYwv10vO40T{gKaoKI62N*+xao+6?
JLCJx0SU}72P%cS0ZUR3ZYk+:03Rzi4g16RxH8Ywlbnw[0br[-03RE)0T7/lAuCB2aP&?*aPRWg0ZE>e
c&%@701w]J0vO0-a]&5LaoA})7:f7+kMTg[0W4K)0T{mMmG+$00ZNI^5fI350U=*U8!{=503RBM0ylYe
01n]paoA})2Qfo(05:(!aoiI+huBc)0sH1Kk[[5R0ZNxjaoiu71oQ[i1r!0)0u}If6LX-<k[E=N04m).
a]@Sa1rWXg6HroM0uk=40STzB4fc+U2setF0SSPJ21n$90ZE>fc&$wO0^3-zIOGjba]%7[03ztk1va7h
ao<j}5jV<*ao&}*huBNn01w#r2[4<Waojq35fH@Z0sPkwaorO+huA>307vt33M]A(1#VJ1aoA})5f.bT
kP*4H0x6#<aorO+huBZ6aoA})6E0MD4f^{Paosw46D^A005:(}3<E7Z04m)K|Axq 1-2p.aoJ.!aoLgV
0uif:aoiI=aorA80W4E^0sO@o01h9BJLCJp1vmFV2x$eC0SSPJ0ZN9baPJR^aos+gAY9$4aorO^y9AW3
0vO0Lk((0:0T5]p3J-NV0W4F+a{74gaoDt0aorO+|tqh"2seDla]$0RI[/sca{5>/ld?ah0br[-03Ry>
0TG$Lm*87110vU60T{gMaoKI62N*.wld?gj03zzgEJO(tAw^<#3M]A(1WJlhhuB1703zCWa{Gd<ao=K8
2sexhhuB1703zqSao%!a072L5ZYk+:03REEkTBQ[ZYk+:03IvAkTB4ZZYk+:03RLi0u.Cf19O=g193:j
kTB4ZZYkFVaoVZy0STtA|AxqA^2kWty9A:61pLqsaoB&NAuUDI^2kWty9rZ50ZUE%ZYn9$0STwC3M]M{
0x6#+aoJ.!huBpf0u.I60T{gLaoKI62N.:Pk)RiG072?dZYkFYhuA>A&eLILao%!a01o5tlemEn072z1
ZYk+:06#5R0W[9&Cx2Ix01e&[0ZE>gk[D/{kP*aJ1WJlhhuB170W4UYa{Pj)k((0:0T6RFao#T712ZhX
leW:r04m)Ga]$aw0yt}eZYk+:03IszkTB4ZZYk+:03RLi0u.ze19O=g2606mkTB4ZZYkFWao(<A0STtz
|AxqA^2kWty9A^70sO#paos=MAuUJK^2kWty9rQ21vpW#ZYn9$0STwD3M]D)05:(:aoS!!huBpf03zq2
0T{gKaoTO72N*.waoAU/ao=H706{ZchuB1703ztTa]&.=c&%Jq0DH711pLqslfKrz03zI80SUKAJLCJp
12ZhX3M]Hp2X>K-mgxtc1pDsLaouHK1vblMaPT>gao({i06{Pc0bDP$32}c?mgxtw0baU1Bv0Z1ZYn9$
0STwz3M]Br4fl?skP*dK04m)K|Axq"0W4UZa{o1<aoJ.?ocKGTaoAU?nG[iPaP&?/aoiI+aoSSa0VJo-
aoi!>5f.eUE&[^@aoi!>6BM8+4fc+zaos[u03zCh&eLILc&%w)08jNraoiI/yFO{V03zp<0Vi6+0ZD/L
aorO=c&%/307E8Raos+xk[Cdtc&$8b02]V30STzC4fn-{aPI]L3M]D#0TG$Kaojq33)kJ)0Yy8=3&{.U
0ymAmaos+ty9iG}aP})jaoS?)aoA.!k[[5R06}fhaoi!>6E0O$aP@[/k[(P@aoAG%c&%w)0U2bqao$gj
huA<^0ZE>jmgxj#kMTg[0vO0?k(#c]Z(2]cy9AZc0STtzm*8710u.N{0Uv<8kP*mN05:()aoh&L3<l}X
1%r[70UuEO4fl>t3M]G[05:(Uaoj?t0sH1Iao<.97:5&c0UuKS1va7faoBC55fI2%05:(}3M]D#0U=*W
mgxnu03zt30U=:Uaojq37Z{AMlemEn072z1ZYk+:06#5R0vO0?Cx2Ix01e&[aoK[vpxS{90u.Li19cp3
1rWZ/1.]jYaoK[gAY9$4aoiI^y9iQp0STtBl4x2j1va7faoJeY3<3!rkTB4ZZYk+:03RFg0u.Fg19O=g
1Au<kkTB4ZZYkFUaoVZy0STtBlhgF2lc$L90brXWaP-5M3M]Kq{YkvU1vpy)ZYn9)0W4Ri/O^JBya6b-
aP&8{0W4F30TG[Jaojq33)kJ)0Yy8*aoiI^huBc)0sH1JaoJ.&y9ATz19cp31rWW!0CS!Uaoi!>1vi6e
huB0?aos+ly9AS!0sF@]0ZD/L0ZD/Lao+6?JLCJx0STzBo9vH51WRjeZYk+:03Rzs1va7nk)g{C00mS)
1WPtx1zQkMFp8tX0ymouaos!d@@Wmb07Q=F0ycSe1vmJb1vblMaPT>e3&{.U0ymosa{74wyAJW31viPv
a]%$faoDsF8W^!<aojXchuBdb03zt30T6RCaosw403zK}0T6XFmfC>w03IsJ2N.:Xc&%/22sm<sZYkF.
c&%U#1vqKpZYk+:0brXThuA>A]O{S^aoj.1JLCJx0STzCaoiI=ob/+<00ky!kTB:@ZYk+:03REEkTAhz
ZYkIT13^$faoi!>03RH/04m)Ka{I^K4fvTLc&%U#01w]o3<3/5@eFF(c&%w)06{Pc03zqp8:%({lhV.T
03zq20SSP]Fr4zBlh@2+0SUKgI[/rn2q#R-lbXX%03zz50SUK4I[/rn0x6#Vlemyl071$&ZYlm]072X7
ZYlWMI[/rJ0SUKsI[/rU^2kWthuA>A)2eTVleWWp05:(UleWWp072m}ZYlm]073mnZYlW:I[/rJ0SUKI
I[/rU&etwJhuA>A@ent<lgx/F05:(Ulgx/F072?bZYlm]06#o%ZYlW}I[/rJ0SUKYI[/rU[qC6ZhuA>A
JeUKLl4B{f05:(Ul4B{f073yrZYlm]06#<dZYlVRJkbAK0SUJxJkbAVFq?nzhuA>AOq+k-l6d5v05:(U
l6d5v06#B1ZYlm]0700hZYlVZJkbAK0SUJZJkbAVN2F/XhuA>APP3U^l70RD05:(Ul9c3X070opZYlm]
070MxZYlV[JkbAK0SUJ[JkbAVSeOH(huA>AU-cu$l8Y:T05:(Ula&e(070&FZYlm]071bNZYlW8JkbAK
0SUK8JkbAVXqXi6huA>AZ(l5elaz(?05:(UlcLq6071zVZYlm]071X+ZYlWoJkbAK0SUKoJkbAV:C^[m
huA>A^2t:ulcb2205:(UlemBm071$<ZYlm]072X8ZYlWMJkbAK0SUKsJkbAV^2t:uhuA>A)2nZWleWZq
05:(UleWZq072m@ZYlm]073moZYlW:JkbAK0SUKIJkbAV&eCCKhuA>A@ewz>lgx&G05:(Ulgx&G072?c
ZYlm]06#o$ZYlW}JkbAK0SUKYJkbAV[qLc.huA>AJe+QMl4B%g05:(Ul4B%g073ysZYlm]06#<eZYlVR
JLCJL0SUJxJLCJWFq}tAhuA>AOq>q:l6d8w05:(Ul6d8w06#B2ZYlm]070AuZYlV/JLCJL0SUJNJLCJW
KD43QhuA>ATC$0}l7<jM05:(Ul7<jM0700iZYlm]070#KZYlW0JLCJL0SUJ+JLCJWPPc.!huA>AYP6Yb
l9Mu:05:(Ul9Mu:070MyZYlm]071L.ZYlWgJLCJL0SUJ@JLCJWU-lA#huA>A+-fyrlbnF}05:(UlbnF}
071bOZYlm]072a]ZYlWwJLCJL0SUKcJLCJWZ(ubfhuA>A)2w^Xaos+s|jAxqa]<[myA-*r0SUKkJLCJW
:C)$nhuA>A<C*}PaoT$UyA-)70u.wTyc-bua]<I503zs=06{TahuB170u.IWk}1R)1vq*xZYlVJFpEUU
0SSSm3M]G[14:QoaoCnT4fdHJc&%/313(%c4fdHMk((cK2sYd83QKYIZYlW#JLCJx0STzAaorO+aot4M
huA>30u.IWaP>3CI[/ro00ky!kM:m]10vN(0STzGm*87104m)Oa]&5M3<3!Aaoi!>3)C^t0DwLGaoK[g
B%42V4fl&5Z(b#daPIQf0ZE>faoi!>03RIv1va7haoJ.^c&%J^a{HQF0sH1Ic&%U$01n(o3QKcsZYkFT
k[vY@CYs]ck)RlI12ZhXlfa3v03zzgc#g1kaorO+yIDcvy9ATo0STtBaojXdAY9$4aorO>|jA(60T7*:
JLCJWFpJdJhuA>30W4Xk| kAxAx%>Ca]&.=aoB?uydPu*13)>:0T7*8I[/rw0STwJaoK[vyc/}0ZYk!+
05<@VaoK[ny9AT23o51^lcLn503zI80SUK8I[/rn1u3qYlbnz]03zq20SUKcI[/rn05:(UaoK[Hy9rMV
kP*4)2q#R-aojXgy9AT221Ym73M]G[14A.8aoJ.*c&%JqESmmE1rW:*10vUXa]<[fAY9$4aoJ.^huA>3
06#bppxS{90W4EO2[d7E0ZG67JLCJx0STzBk(&8dk)8pLa{q)O1vbo9JLCJp0u.I=huA>3073wXAy4%f
ZYn9[03IvK0sH1IlhgF2a]<{QI[/scaPSX=ld?ah0br[-03IsIaorO^huBdb03zt30T{gMaosw43)kP]
05:(:3<Nd.03zq20STtyaoi!>1rW?ZhuB170ymlh|AxqA2X$Q!ao+4kAY9$4ao>ax|jAxqk)RlI10vZ?
1.]jYyAS<713#pkZYk+:07Ez1aoK{[JLCJx0SUX01rW^{0T6XGk)89)0y^hSaoJ./|Axq"0t2IV0u.LX
aQ5$?aoMv511jiNaP-2]10vUhESmmE1rWW!1WJffhuB1703zCWao:U803zChFc5HHkP*4H1Rp$G1}7F$
ld?gj04m)Ga]%$gao+4kC61]fAyy0}ld?gj03zp+0=*g203zCh{Ykw2^2kWty9A*80$khrao+7a0Dxp:
lc$L90brUVaos^XI[/scc&%w(1Q=BPaojq32P%o[05:(!aoiI^huBpf03zC60TGhr3QJ<kZYkFSao=K8
0x6#Vlf%PD072$hZYk+:03Ry=1.]j.huA>30W4If0DH711rWW!1WJlhhuB1706{)9aPI]P3QK0oZYk+:
03IszkP*7I1-2p-k[uV)kTBQ[ZYlU2huA>A&eLILk(-!A03zm:0yl*dhuB1703zqSa]@!!c&%Jq0DH71
1pLqsleW:r03zt30SUKIJLCJp03zCWa{o!903zy!0ZM<chuB1703zqSaoBC503zm:1WJlhhuB0?aojXk
y9rM=1Q=BMaoS!>ybMa@lf%PD072$hZYk+:03Rze4@ayPAuUAH2X$Q=huA>A<C*}Plfa3v04m)GaoVB6
1rWW!0C=Bo2X>K:huA>30W4If0DH711rWW!1zQjJhuB1E{(i5?l4BGo0x6#V3<c(6)2w^Xaojq3072X9
ZYlWYJLCJx0STtDy9AWp0STtyaos+eAY9$43&{-4>-9vTaojq3072L5ZYlWUJLCJx0STtDy9AWp0STty
aos+eAY9$4aoiI+y9iKn0SSPJ1virdaPI]K3QB+o072X9ZYk+:03RB^1X/Balfa3v03zp+1-2p.huA>A
)2w^Xlf%PD04m)Ga]&.?y9AZq0STtAaos+eAY9$4aoiI/k)8c=0T6RCk)RiG01f[Pk[E=R03zmHLM?c(
E&[.S2X$Q+aojXgyBxk:a{gbQAuUx$aP.Zg0ZE>gk(>%m03zwf0=ZTr0u.y>0STzDaold103zp+18x7Y
lfKrz04m)GmG+$00W[9Mk)89)14J!3leW:r03zn10STtAaoA})1vmGRhuB170u.wd0DH711rW:*05:(U
4*%GXaoJeX3J-M{kMTgLkMTg[0W[9Ma{gajAyy0}aoB<}JLCJx0SUX00W4Lg>-9vTc&%xr4fEZOaoK]O
AuUC}2P%c<03ztTa]<[fAY9$4aoiI+y9iHm0STtzlfKrz04m)Gm?2F+&eLILaojq301PCW0W4OhESmmE
1rWZ/06{TahuB1703zqSaojq301f[Hl4x2d4fvTMaoh@Vk(-5el4C0h06#c}ZYk+:06{T3a]<I503zmJ
1vpa!ZYk+:03RyCkM:m]0yl:3aPSX=c&%U#01w]o3QGr6ZYlX2a0pOb0yu-co=C$(01PD6)2w^Xaosw4
072X9ZYlWYJLCJx0STtyy9ATo0STtzaojXdAY9$4lfKrz04m)Gaos}mkTBs/ZYlU2huA>A>-9vTk(-!A
01f[HlhlCP04m)Ga{gKG13#pkZYk+:03RFk4fF(@aPK*?JLCJx0STzEk}bwm0ZTRWZYkIU13^$gaor>(
03ROx1va7hao->*c&%J^nEU!gaor>(2Q6iS01e=mlfKrz03zq20SUKAJLCJW&eLILc&%w>0brXThuA>3
0u.wd0DH711rWW!0CS.ShuA<?3J-NV/P0VDc&%w)0ZM&f06{ZgBrQ*bmfC>w/P0VDaoAU/AY9$0aoj.5
0DyALI[/sca]&+/3&{.U073wXAuUxG^2kWty9rT3072m}ZYn9$0STwz3M]G[0x6#+aoiI+huBpf0u.C4
0T{gLaojq32Odqe:CW?lc&%w)0r+T&kP*4)0CS+Taor>(2QfoU01ff2Fr4zBlh@2H03znc@#T@OhuA>3
10v.x4feY?JLCJWE(mud3K.[30sKwUkMTg[04m)Oa}l?*FpLAoAyyrp0ZD/L0ZD/LapQR!FpJdJArFJH
051u.a{ntQk(-5f3<3!V0ZNw[1v93r0ZD/LaoAU^k)7</D#Qtlaoum414-}4aoAU/yA:cL1w645aorO*
m*8710u.LYa{ynWo:0Y-kP*gL0u.X-a{GN#071wHn[]EMk(>ZK002s1E[BC#20&r9ec2VoZKF<<ao>al
yaGt+lbd[wy9rZ52%!Yea{!XU3KX6Taph^a2M)sm1rW$3002s1E[BD01rW$o0CS+-ao+4iy9A/<01feS
0u.FVaQ4{japyEqAuUPQ4fdHOapyHk%nS97ACuW1ec2VoZKF<03lPKs0y^H#apoj[ec2YpZKF<<aQPs@
k(%H+03zQ>11TANlbd[wy9r](apyEpB%dnp1rX4^aQFloao-[<apHWF0ZTQ5apGw3lbiH%8ZkIM192E>
0ZE>kk(#cL3QKm!AuUMP1vblMaQwFS3QB+o2P%i)1oQ[i2P%l#0STzGl4r@w2x?i]k)zHMl4KSv2YTP4
1vif9c&%w)20)-Wk)IMQ20&DpAZtndGA#NjaoT$oya6b-a{Ht.B3Q4Oao>aoC5/)1F}KzHy9iTB3#df(
03RRkE@^)XB%3#P26L2MFR5KXACuT5aoT$wy9rZ51WJYna{wzQ3KX6=ao#W93)kS{3U***ap7msC0VdV
@0di:2TOGw@0djRl4KK?y^wRQaph^a2{oNz4fc+qE&[?o2snm*Ax%UFy9A^e0STzDl4r@w2x?i(k)zHM
l4KSv2YR3(appynmHYtoao-><c&%I}13}ATk)IMQ13)cmAZtndGA#Nja{e}{k(%H+02=%ZaoT7]2Qfvp
E@^)XB%3(M26L2MFR5KXACt>913)iolh#GQAuCuH%ag7MACv^<FcoWO5oYf5y9r((0sIf[aoB$v0u96Z
k)89I1T0>j1wYd=aoB&RAuLz^kP*vQ0u.U.a{oB%071wHn[]EKk(>ZK002s1E[BC#1vi97ec2VoZKF<<
aoT$jyaGt+lbd[wy9r)a10v>n1zPa^m?2Ca3M]Qw4fdHKap8Z91oQ[i2{ov4002s1E[BD02{ovp0CS+X
ao+4iyA-]>01feS2[4)Y3<l}X0.0kSk(-5f3<l}X0ZM{daQf4<k)f%^kP*a]3#)A)13^$laorO*y9A^i
002s1E[BC#1vi66ec2VoZKF<<aoT$iyaGt+lbd[wy9iTB18ov!071wHn[]HOaoJ.&k)g{D1Xf143KX6Y
mgxtw0u.LXaP?*iao%1>ec2VoZKF<<aQoa(k(>Zz10v!l0C->-4fc+A0$cjLaoDp33QB+o0W4H!10vZ&
1P}1j10N{(3Q$NbaozQ]aoK[gy9iQf001hAk(:T?02=%Zk($^z0ZQi=4fdp?13)37aoB&FnEUUraoK[j
y6#i/ao(Q@aP-+&k(>:B1Q$Ct3QB+o1Q=BQao=K72mzTVaoj4}3)C^H4fdHMao#W91vi2h2setk1P*{<
kMTg[4m81Tk)89)0C-eD009ceaoS?>3&{.U1vr4!193QcB%d1%apQTp%nJ7saQf4?c&%I{3lPr%0STwK
13^$lli3@!AuCAJ%nJgvnD.pA0u?K/2sex9aQoa$ao&}}c&%}(002B%0sP5r3QB!p10v%]0u.B^3mC@:
5DR}w0ylYe2oTc(1-3B7@@Wl:0rUOf2sny&192H+aot4S10vN^0z:daao$gky9r*83M]S@3mC@!5Dz/1
4fc+A0sH1Ic&%w>0u.B^04m)Kc&%!?0STwC3M]JVFck(UE<j}V5nJD]bME)}0ZE>gk(>%]1va7heDt+{
1Q$F#01n]paoiI=aoK[gB%3/l0T]Ux01n(oaor>(3)t@WkP*a]0CS+X0ZD/L0ZD/Lao/LD07WkTao+7h
0Dxp?l4w#94fmNSlbiH>4fEZSaosw41rW)>05:(Uao>f9FpJdOhV:mc10v?m19wo40W[9Iao&}&c&%I-
002B%0ZM?}1}8KNaoS!<lh{$9a{5>]5DR@31va7haoVB50T]htk(?w$3M]A(0ZM{6a{o1&f8#1%0W4*Z
0SUT(kP*aJ1zP7Y3<c(60sP8sao$gky9r*81r^*P0sIh^FpJdOaQYy@k(>%dkP*aR00akJaoB?jy9rY/
k(-5m0!hr@k(#3{1va7jaP:}f3&{.U1vi97aP-+?f8$$WaP@[>k)hfgE&[!C001eGaoB?gy6tZY3N2=#
1WJAl1B?K1f8$$}2oTpr0+}(Y2N.^Sao+4xArFJH10v.j%nJgvk)8jzf98h01pDsRao+4NArH+l10v[o
19woh0r-MG2[5/}4J>@#1uMR)ao&}@huBdb1%r}80T6RIaojq306{Sd10v?m19wo41sKrKao&}>c&%I-
000Df8xYnJ0CS+.3M]H5001hD4fc=401f[Nk[E=R01j?<0rN-[03zqSaP.Zg0ZE>dc&%I}13(%c4fdHM
k(#3{4fmNKc&%w)10vRVaPSX=aoMy6072?dZYk+:07E8RaoA})1vicfk)6Q^0ytU6ZYkFThuA>30W4K)
0T7/rAw^>3aoiI+k((0:0T6REaosw401n]paoiI^5c@TBkMTgLkP*aR0T6XHk(#3{1va7hlf%PD04m)G
mHYzw0ZV4fZYk+:07Ez1aoAU/|Axq"0UtRW03zp+0+@[Wk((0:0T6RCaouj10x6#VaojZ>JLCJx0SU.1
0ytU6ZYkFThuA<?3M]G[13}xQhuB1703zqd0DH711rWW!0CS.ThuA<^aos=U0-5}:aoiI+5d5<P0ZG67
JLCJx0STzCk(&8ek)8pLa{h*N1vbo9JLCJp0W4O=huA>30yuFYAy4%fZYn9[0u?HM0sH1JlhgF2a]$0R
I[/scaP-+!ld?ah0br[-03IvJaoAU=huBdb0u.w20T{gKaoBC53)kJ)0x6#+4*$X&JLCJp05:(Ulfa3v
072X9ZYk+:03zqSa]%O603zm:0yl:bhuB17072?dZYk+:07NF0leW:r06{Py0SUKIJLCJW05:(U4*$X!
JLCJp05:(UleW:r072L5ZYk+:03zqSa]%O603zm:0yl:bhuB1703zqSaosw401fg$0S-EO0ZE>elcYBR
k[CdsaojXso=CnRyFLxtaojXsaos+o|jAxqaos+onGe&Ryc-ny5cs3Smgxkt0ZN9caPRTfaojXdyA-<6
0=ZTikP*7J01n(oaoB?iyA-{g0STzG|Axq 0W4OWk(-2cyJhSCyA-*506{Pc0W4IVk[D:fy9AT20C->X
yAS^520&uh1va7faoJ.^c&%Jq0DAcC0=*g21rWW!18o1ZaoK1[1vi6ehuB171T0<?1T<ALk(>$Ck(#6+
0STtzaoDp410vW]0T7^&AY9$4aorO^5c-Kz3M]D#0STwAaoiI^huB1703zp+0+%Q{01e&[aoi!>1r)>j
193:s03zqdCYs]eaoT$wye2cqaoiI!aos+eADjtDAY9$4aoiI!y9AW30W4RYa{pglAY9$4aoiI=y9AZ4
0W[9Mk((0:0T6RDaoSkW3M]Bn2X>H+3M]JVF==2RE&[.t0T{jN0ZD/L0ZE>el4x2j1va7fc&$ke0%gZ?
kP*4H0.9qTaojXwk[Cdsc&$8b0VL(50STzB4fn-{aP-5N3M]A$0TG$JaoBC53)kP]0x6#+3&{.U06}rl
aojXsy9iM%aP})jaoS?>aorU/k[[5R0ZNxjaoA})6E0L%aP@[?k[(P@aorA@c&%w)0sY2pao+4hhuA<^
aoL4w0S>Rf04m)*k(#c]Z(2]cy9AWb0STtym*87111jiZaoj>u0u.E!0Yy8]aoz#O3<l}X0u.C40STtA
mgxwd0T*aMaoBC55fI2-0sP2qaoi!>2Qfl>0.iwUaoiI=huBpf0W4F30TGqAld?gj072m%ZYk+:06#5R
0yl*hCx2Ix01PCW0W4O60Vi3Wc&%@70r+Ug0W4I40UuEPaoBC57Z?C^c&$8b07vs%aoAU=huBNn03zt3
0VhBQ4*$XUJLCJW*(o8Hc&%xmEJORs0VVA7huA<^.#Y@xE&[.T05<@<aohE)k(-2dl4x2d4fdqja0pOI
%nS97oap>taos+Paos+lC4S03yJ.#AAuCuH0D-9yk57DH0Yy91aoB?gBv0Z1ZYn9)1vi5h0^2Q*lemEn
04m)GAyy0}aoS!!huA>303zz50Vi3Waojq33)kJ)05:(:lemEn072z1ZYk+:03zw:huA<?3J-M{kP*7I
1sKrKa{fj{1vmoLmG+$010EZO0sH1Jk]y^Dk((dAk(-2ek{5PoBrH!&kP*dK1WKeLk)hgxa{op}5f.fA
4fvTQk((6J1T0<&10vT[0T7/lAuCoL4fc+zaoA})2Qfo(05:(!aoAU=huBdb06{Py0Vi3WaoBC53)kJ)
0x6#+4*%G.k[E=N05:(UaoiI^huBZr03zn10T{gKaojq32N=B&0s12@04m)Oa{o4<0!iGpaos=U0z9.2
8BjAE0yqkEnEUUrk)6yQaos=UFcnB.3Nb=$04m)GaoVEM1B?J]aoS!>5dY(-c&%UTaoz][04m)Ky9rSW
kP*7[Fb@BGkP*7[kxe#tE@VXoaos+jB%c@g0yqkEnD.p30W4Uc00ahzaoK{B0DHj301n]paos+pB%d8k
13]d>l4BD9aP&?/li3@!o:0Z70W4Uc00jnAaoKU80u.B^2smnGAYKg23<3!V0W4Uc00stBaoKU80W4K!
2sgN]l4BD9iSGg60W4If5{uMWAYKg23&{.U0W4I8000xH03zy!25l3$2TFB#HYNOYE&[.t0TG$MaP.Z{
k(&8el4w#c4fdqj0W4IfFdLoY02!c9k)eqhl4BE*nGe3Y1%r)]0STtCyFa!WaoiI!ao&w?aoi!>2N8sL
3M]A$0T9lY0S>Rf0yqkppxS{90yn}&l4BD9aQ5$?k)zGQ10vRgFdLoPkP*aJ1V=R:aoAU/lbiIwiSGc*
0T*aKk[4]W2oTamkxe#tE@VXmaos!d@@U%#kP*aJ1V=R+aoAU/iSGg60W4.l?#N<V000Ae3M]G[1V=R=
aoAU/iSGj70W4.lkxe#tE@WNI0u.B^0ymuw|Apr:000Ad3M]G[0xHhX3M]A(1rW[-huBdI01nS70T0]M
EJ[?UfL.<lbMF}/c&%xmFpJf2CT.*3aor>(3)tY}0ZOwLa{pgihuA>30Z.A/FpFg&0WF(&|A$jAXp?S$
aoK1[03RH[0STtBc&%I.1SwG+|A j 1sKrKa{f.803zs=0X1f@a{x[c7:5&B2X>E:huA>30u.L80SSPJ
0vX6IaQ5$?lv:PpFdKTv03zte5nAx>aos+ly9AWb0SUaW03zqd05:(UaoAU?hV<sK3{+1cg9M-/1vblQ
k[1q.01f[IaoA$]2Sz!+aos+ly9iQb0SUaW03znc:CdFghuB1703zq20STtA|A0j$01k1)0rN-}06}-y
a{xj(la&8<071nPZYk+:03RRk0CTH]00ky!E)B5S20&kW4fdqj0yrVwZYk}^01w]Dl8YWR06{SD002rX
IOGiTQ(8}/c&%xm0CTH]06{U]lh{$9a{Hsmm*87120&ofmgxnu1WJAeaoiI+c&$j$0rrJe3QI1OZYk+:
03RRk07WL0l9MlZ03zFi0CTH]070#HZYk+:00CLf1T0^/0vO0-5DI>O1T0}e039f=aoKU893tzc0Yy8%
ao->>dfxMe0UvV8IOGiu0STtDk[E=%XqF64c&%w%0U=jE01h8%IOGiTU.{i@c&%xm0C:N{070MvZYlU2
iSGd514rU001e=mOb&S(E/.>pkMTg[0yWbRaoj?t0u.wd2X$Q+c&%xm0y)O2aoi!>2P%9}0T6UFaos+d
huA<WkP*7[E)%>EaojXgyA-:304m)Gk(>:B05:(Uaoh&Jaos+B5hazl0.0L2aoA9c3<c>W07vs%aojXk
yA-:304m)Gk(>:B0x6#Vaoq]Maoh@*4*@}901h9WG1}vLkm/QJ3K-0%0TsbPEJ[?U5nJD(bMF}*aorO^
y9AWK1vblMk(.Dw01f[Kk)g{C1rWW)0T6ULk(&bkk)etj0ZG3SaorO+c&%w)2sexja{x7&ao+ySa]@!!
k)Qg7a{C7)2%]+<p^>HLk(-5e3&{.U38X%Fli3#+@%8g1kTtw[0sO#p0ZD/L0!hr$1va7nao$gkaor37
3&{.U0yWbRk(&bg3<3!V0yl.$g7boc4fdHNk(&>B1pLqsaoS!*huB1E03IKOk)ORn3M]G[1zP4WhuA>3
1rW[b0STtBc&%Jq0y^hSaoK1[2P%i$0T]R=01f[Kc&%U$0u.v+1VuzZaoiI+huB1713)GkbMFjx0rJvM
0E=l[|A k"0WE:*0vO0Hl4BGoFoTJ^kP*7Q0T{jOaoB?Gy9A^D05:(UaoB(WFpJdYhV:Wo0ZNJnlazP-
03zv(0STzDc&%w>11jiN5cS%{0ZNxjaoT7]03RI60STtAaoA$[6E0Yt0%*1Ik)RiF12ZhXaorO*hV-%!
aojZGHqi^E0T6RCaosw403zteazJe43?jk#1B?S@k[E/Q10^<?0W4H^0+@[Wo9vHC06{O@bMFh{13)68
aP@[/c&%I{1{fou2P%c<04m)Ga{74hBrQ(h0u.Ls8Zkq90yl#=8ZkCf0ylYR1vblNaP:}faPT>i3&{.q
E/.>pE&[^}kP*mN0ZM&f0t3:&0sH1JmfC>w0u?HM0T*aKk(?JL3NbSY03zzg0x6#Zk(?w{3M]M{0Yy8.
k(.oo0ZN81aoVB40x6#VaoS!*huA>311jiNk(<BTkP*dS0TG[Jc&%!*bMFh{11jiRaPSX=ao:U803zm:
0x6#ZaoK[vy9S=!q2{YEblg1LyA-^803zqd3n}}]aorO+huBdb0u.ze2X{FdFpJf2fOzv!7:5&40ymce
TWmL*FpGtbhV<QSFpLAnaos+tyc!8uZYj]:01fztblg1vyA-:703znc5nAD<3R7i#E&[.t0STtyc&%U#
18x1XnD.p303zv^0Ut>+04m)OaP&8{0T7+h04m)KaoMv30u.Ew3ig5laoiI=aoMvp0TI6(3R7i#E&[.t
0STtyc&%U#18x1XnD.p303zv^0Ut}^04m)OaP&8{0T7+h04m)KaoMv30u.Ew3ig5laoiI=aoMvp0TI6(
3QsZ[E&[.S3#df(03RBDkP*4)5nAP{03zpNfad?DaojYSmHYtu03zm&0T7^&yA-^q0T6RD4fdHJk]pd^
3K+.{0T%LVEJ[?U5nJE0bME)}aorR?apQQvy9r^E3lYD[04m)Ga{/B[lfFLH1va7lk)g%D1rWW/0rLH/
kM:m]0u.zeKJ)&za]<]<=h.V@a}3Zp@@Wmb<n:6v2M^ml13)oea{{QsyE5q01va7jk)?uJ2oTpr0DYH]
eDzP5kSO(BappypyA-*C3p$WcaoA8!01f[Rk)g%m5=-pZ2sex9apHMjH}<19eDt=i001bGk($*/3p$v=
ao$gly9i)b2TNIKy?(rqBv2{a2xp{?eDzP5kSO(BappynyE5q64fvTSk)7<A0ZSVsZYna30024W03zzg
1zYd-aoK[jyAS^50yu:!&SI)%03Iwi4fE1t3KX6:k($*k5=-pZ3paYd5e1%JaoK[py9rS^0ZE>dk)Zl*
kP*4I10vT*0sO#paoj.c@@Wmb<n:6v0%g.g0ZM)6a]%$pnD.p30u.OYaoiI^lf52ZyEe12193QcBrQYp
0hLAoiSGd50ZM<5a{74qpyNsE0W4XZaojZ7H}<19eDt=i000Ae3M]DY5=-pZ0Ut$A3QB+n2{or?mfC<#
0yl:4a]%$ppxS{90t2>z3M]D)25km.k((6V0hUGpiSGc/aorO?y9r%J3lPv+aQyvlaP$7iaQgj.l4BH7
03zB[0TG$Kl4BGo0Dxp-8ZbRj0ZRu!Fco<XmiI<pkP*4)6)pY9y9A:51UP(+a]>gqkMTgLkP*a]FpJdQ
Ayy0}aoiI^yASWz03IvAkMTgLkP*a]9D<ZJAx%Rv4G=Xi00iJJ03IvK0sH1Ilh{(^Ax%RGaPR@[0ZV>^
@@v3Z0W4T{0T6UMao:d{03IBDkP*g}%nJgvaos!d@@Wmp4fw/@aQoa)k(>Zz1rW^?0W4^$0Ut}z07vs%
3K^dCao->>dfo=04YqQyFpNVBAy3*VFpEYwhuBdI0u?W<1T<ALa{5><c&%I}0u.^[1}R:-10vN^18yj5
@@Wl:00tFe1vr7/192B-pyNvF1vi66aP@[?k}<zZc&%}(002B%01fhq3QB!p2oT9>2{oT01}R:-0W4N/
3M]Y$2}b&-5DR}w0ZM/f1rWW!0C+13@@Wl:0rUOf1vr7/192H+aot4S2oT0?0z:dbaoT$hy9rZ510vT/
2}b&^5Dz/14fc+A0T*aLapxp$aor>(3>SKv4fmNPapZsi2TFC00T?o{aQoa[c&%w)0u.K]0T6XEapGv%
5f8bVaorO)ap67&c&%!?0STwG3M]VZapQQFy9S=!lJEjqk]q(<5hCBhmfC>w1vi>ohujac0W4I40U=:U
aojq35fI2%0x6#/aoB?ehuBdb0Z.D*FpFg&0STtAk)Rikl&^vsaor>(1r^*?0vO0HaP>1ok)d-Oa]$an
kTtJs2On(]3M]D)0Yy8.aorO!huA>3071bHZYlm]1rWW!0x6#V3/g)=0E>c0blg1LyA-<a03zv^0x6#<
aoJ.^huBpf13(@H0uri:aoBC57:5]613)ughuBNq06}fia]@}?aoK[zy9ATb0STwDaorO+huBpf0u.I7
0T6.Fk[E/Q03?K!0yl<6a]%7[03RE)0T6XHk(>%dkP*aR0STwBaoiI^k((c!0T6RCaoBC503znc^1Ggm
aor>(1rWZ[0TG$IeDu5004!fS5dFQEaojYTFpJe{huA>303zq20T{gKlf%ou03zp<0T6RDc&%U#04!fR
aoi$[2[NKC3QaLk04m)Gl4BGoFoTM!kP*7I04m)Kaoi!>2Onx{3M]D#0STtzc&%I]04m)Sc&%w)04m)G
aoi!>1p$.Ah#<1BaoB&VFw.BS4fdHJaoAU^c&%}(002B%06{R@3M]Ku1vblM4*%GWaoK[faor>(3>SKv
3NbMLkP*4H0t3LC4fdHJ1va7faor3ea]$aw0sH1J4*@vB9@(vcaorO!5g4DAaoiI+aoJ./5gX}84fl?p
3M]AS]92d5E<j}V5nJD%bMG4?|A k"2p6oJkMTgLkMTg[00CLf06{)aa}2I3c&%xm0CS!WhuA>30.0L0
aoi!>03RFgE)%>GaoiI=k(>ZU0STtFaojXgyoo*MFpJdYGF@#Iao$gvy9rZ806}fia{xj(0ZD/L0ZD/L
0ZD/Lao$gHy9A%I0DwlcapgsmB%c>{0sIiMG1}vr001hzmgxn-&!1@CaP?*iaoK[gy9rVXkP*9yk(:BU
kP*7[@@Ea9Fb@axkP*mN11%SOa{8H51%r%&18pa6y9rV/0T*aMaos+gAx$B=a{74lBu$CcFpE=xaoB&V
FpJdKAuCrGFpJdQAx%?KA=KC^C4@c2k((c{0=.@$0yl^gk(#4vaP&?>miz>RaQwFS3M]J]0yu.^AuUE0
aP&?*ao(N71}8KQeDt+{0sY2pk(-5eap67]k[DRc4feV]aQxg{k((6K0ylYR4fN4Y0sY5qk(&bhk(-5e
3&{-40u?Z>0yl.$g9M/?4fl?V1WJbD0TG[LaoKI61rW<<0x6#Vao+7eHqi^P&!1@CapeO.mgxnx06}-y
a]&>*aojZU05:(YaojZ6HRJ)F0STtyl6Neu05:(!aoiI/k[vY*0TG[GaojXkyoo*MFpJfi1b5Pn7:5/3
0iFOrFpJfy1b5Pn5j9^QZYkFSk[E=%@d&5/5e>JRap67<5f}k-aoS!<dfoGd0STtCk)RiF1WJAec&%xb
0STtDk[E=R03zH{0T{jUao%p@5fR8$2pGS/aP@[/aoi!>06{T3huA>33M]*90SUH!yA-:p0STtymfC<#
3Lj6Y0ZE>hapg*DkP*aK01n(oaoUaokTtz]03zs=2[O-#0sH1KapgsmaoSlaa]>4v1pDsVaoTO71rX3]
05:(Uao$gPy9S=/1pC+$3JHM2:aIeclc/Jxgxm!G0u.HRbMFh{3mC@QapoH#1rX4q5nAD<3NC/OE&[.t
0STzBl4BGoFoV92FpJf2CT?)4aoi!>1rWZRfaed70rEV)04m)Ga]}*iaoi!>1rWZRfaedf03znc2X(U#
G$)Wq0SUdX03zncN1-rQdfoGd0SSPI03znc2X(U!G$)Wq0SUdX03zncH>SRAdfoGd0SSPG03zmAkP*4H
0t4c?3QJB1ZYlUBlf%uw01Z4G3L-T:0CK?2mJ.W%FpJf2CPW35o(QKH03znc?2FL#5e>JR4fdHK1va7f
aor3b3KWZIaoi!>03zp+0W4E>0T6[M3>SKv3LSN-04m)GaorO+c&%J10T]Uv01ftraorO+c&%I]04m)O
5cJx#1{1)S0!gnp0ZD/L0ZD/L0ZD/LaojXgyA:0i0STzI|Axq"1vieR2P%ys192H*8Z2e@pxS{91WJbh
0yn8Fa{Gd)nGe9t0ZE>fk)Zr&kP*aJ0$UXY0U2e@01ohxk(-5faoK{N%9ExF2TG1A13)rf|Axq 13)q^
8Zbk806{)aaQf4)mfC<#21oTnl4x2dAV+Gc0C:#KFcX^<0u.IvA=IP]aoh/S3M]M{25kp+0ZE>eaoUsu
kP*j@)2w^Xc&%xr4fn=UJLCJx0STtDm*8711T<APa{ZEoAsAi%2TJYPa{Yp]y9A^60z9.bao->[5c%xY
aoum41WJX}1va7maorO&c&%xm0DAcC0=*g203zp+25ks.ao+4kAY9$4aoS!>y9A^61sKrOk((0:0T6RD
ao-qX3<^p:2oTc(2pGSNk(>$Ck(#6+0STtCao(N80u.y>0T7^&AY9$43<Wkb&eLILc&%w>1zPa-aot4y
2M^ml1rW.Xa{ymyo:0Z72oTpr0Dxj+AZo+EhuA>31rW[-a]@!!c&%Jq0DH711vi2h1WJbi0sO#pao%1&
ap7mlADjtDAY9$0aorO?y9AW31WJffhuB171rW[-a{o1(huA>31rW*}0T7/rAw^>33QK0oZYkFThuA>A
&eLILao:U801o8uaoS!/yA-)E4]0Gqao%1&ap7mlADjtDAY9$0aorO?y9AW31vicghuB171T0%%0T7^&
AY9$4aorO/5c-KE3QJ<kZYk+:03zzVa{o1?oaq1j1}8KQaorO=aoLmQa{d%kaoAU=aoRJG000xH2pGSN
a{gbQAuUSg0yl&O2P%jn192H!8!}}N0W4Olao&}[pJu)GlfK3r0700bZYj])01h8>G$)WO/Od9v5gdw+
lfK3r0700bZYj])01h8>G$)WO/Od9v5gdw+ao%1&ap7mlADjtDAY9$0aorO?y9A*71rW.Xa]%$gAY9$4
lfa3v03zq20SUKMJLCJp1VuzZ3M]Tx4fdHJ3<c>W0$UIT0yWB#aoK]Sl3KHSc&%w)0ZM{d8Z2hF|xqj"
0W4K!14%NS0T7+h0u.v+0@Jx901f[IaP-2]01Ykuaoz]U696cPlazV+05:(YaoiI+huA<^5c9*Naoi!>
03zm&0T65.3LipX0u.v<0STtyc&%I.1pFAm0!gnp0ZE>dk)g%K0STzC|Axq"13)5Q2P%gm192H^8Z2e@
pxS{90ZM/e10vRgcU=o[4fmNK5cAsx3QH.BZYlWMG$)W1ciaAt>.l!Ll70zx01ZES3KWEBaojZqGUNNg
0u.BP1Q+?vaoj.9Hqi^i0u.BP1Q=gF0!hr[k)Zr&kP*7I01Ywz0sH1I5crjE2((7Faor2N01fhnl9bZO
06}Lngxm^>03zm:0vX6IhV<3/82$=Vaos^fIOGiu0STzAk(@ee8XA8J01fkoaos=!H}<0R1Rr6&4fdHK
lf%Dz06}Ck1pCCmaoiI+y6$sFaojXchuA<^3Zz5306#c=ZYj+82oW.#5|dex out of bounds: the 
len is |..B0emh2| but the index is B0el#ZYDLtZYng+3|brary/alloc/src/fmt.rs|A=Rj%
4|ibrary/core/src/fmt/num.rs|.A=Rjlm|usr/local/cargo/registry/src/index.crates.i
o-1949cf8c6b5b557f/wasm-bindgen-0.2.105/src/externref.rs|..................e](1k
6|library/alloc/src/raw_vec/mod.rs|.....055wn7|st/deps/dlmalloc-0.2.11/src/dlmal
loc.rs|...e](1k4|library/std/src/alloc.rs|...02ouZ2|mory allocation ofz!pcc4iPb9
|tes failewN>=B|HellzY(*@aPL})D|tempted to take ownership of Rust value while it
 was borrowednull pointer passed to rustrecursive use of an object detected whic
h would lead to unsafe aliasing in rust|...................................B-RT8
0ce5J0aV6f0dK$G01/hh0ce5J0aV6f0e{)T01/hh00Ao401n&c00Ao400Ju500SA600-G70000000&M8
00Ao400&M800@S9015Ya01e=b01n&c01Ybg00Ao401w]d01F#e01P5f01Ybg0bV2+!*%U-v%yqps8/gq
XS9{Os7.X!u&vOE37ib#+3/vj8|ertion failed: psize >= size + min_overhead|....wNP9H
0qDK904E%G0i(Ub00@S90ax}&8|ertion failed: psize <= size + max_overhead|....wNP9H
0qDK904E%G0jJ7h01w]d03.HQ02LXo0aV9g00@S90000000&M800Ao401/hh00Ao401n&c00Ao401]ni
0aPCR1|acity overfloz*8fY0m}hZ03zmw02$$s00Ju5022tj01n&c00Ao402bzk02kFl02tM2Oqj(W
3*mS)0000n0001cj| formatting trait implementation returned an error when the und
erlying stream did not|.................Bn#q701YbE0001R0SSig0000/|rror0001101YbH
000120SSi70000MR|001020304050607080910111213141516171819202122232425262728293031
32333435363738394041424344454647484950515253545556575859606162636465666768697071
72737475767778798081828384858687888990919293949596979899RefCell already borrowed
|.................................................Cv+p+l5ZYp01e/g0c!/?|roducers|
0TJQJ|nguaxjR8D|Rust01r*!|ocessed-bvTe4V8|rustc%1.93.0-nightly (c86564c41 2025-1
1-27)|....gb*S^|walrB-N:v|.24.gZ[Op|sm-bindgewO}-b|.2.1fF@&T4%hl21|get_features|
2p*s41|utable-globaly&.9u3|nontrapping-fptoint+|..3U1l{|k-memory+d*Qig|gn-eCYUCD
2|reference-types+|.3t<{4|tivalue+|4}oV$||k-memory-opt
