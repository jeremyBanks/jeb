#![cfg(test)]
// spell-checker: disable

use {
    crate::{
        bi::*,
        testing::visuals::*,
    },
    inline::InlineSnapExt,
};

#[test]
fn test_zig_zag() {
    render_d1d((0u16..=14).map(zig_zag)).snap(
        r"
             :   |         .       
           :     |        .        
         :       |          .      
       .         |       .         
     .           |           .     
   .             |      .          
 .               |            .    
.                |     .           
  .              |             .   
    .            |    :            
      .          |              :  
        .        |   :             
          :      |               : 
            :    |  :              
              :  |                :
",
    );
}

#[test]
fn test_signedness() {
    render_d1d((0u16..=14).map(signedness)).snap(
        r"
.                |  .              
 .               |   .             
  .              |    .            
   .             |     .           
    .            |      .          
     .           |       .         
      .          |        .        
       .         |         .       
        .        |          .      
         :       |           :     
          :      |            :    
           :     |             :   
            :    |              :  
             :   |               : 
              :  |                :
",
    );
}

#[test]
fn test_spiral_square() {
    render_d2d((0u16..=390).map(spiral_square)).snap(
        r"
%ZZZZZZZZZYYYYYYYYY   |  %%%%%%%%%!!!!!!!!!$$
%SSSRRRRRRRRRQQQQQX   |  ZSSSSSSSTTTTTTTTTUU$
%SLLLLKKKKKKKKKJJQX   |  ZSLLLLLLMMMMMMMMMNU$
%SLFFFFEEEEEEEEEJQX   |  ZSLFFFFFFGGGGGGGGNU$
%SLFAAA99999999DJQX   |  ZRLFAAAAAAABBBBBGNU$
%SLFA6555555559DJQX   |  ZRLFA6666666667BHNU$
%SLFA6222222258DJPX   |  ZRKFA5222333337BHNU$
%SLFA62iii::148DJPX   |  ZRKE952iiiiii37BHNU$
%TMFA62i...:148DJPX   |  ZRKE952i...:i37BHNU&
!TMGA63i...:148DJPX@  |  ZRKE952i...:137CHNV&
!TMGA63i...:148DIPW@  |  YRKE952:...:137CHNV&
!TMGB63i::::148DIPW@  |  YRKE952:::::147CHOV&
!TMGB63ii111148DIPW@  |  YRKE95211111147CHOV&
!TMGB6333334448CIPW#  |  YQKE95544444447CHOV&
!TMGB7777777778CIPW#  |  YQKE99888888888CIOV&
!TMGBBBBBCCCCCCCIOW#  |  YQJEDDDDDDDDDCCCIOV&
!TMGGHHHHHHHHHIIIOW#  |  YQJJJJJJJJIIIIIIIOV&
!UNNNNNNNNNOOOOOOOW#  |  YQQQQQPPPPPPPPPOOOV#
$UUUUUUUUVVVVVVVVVW#  |  YXXXXXXXXXWWWWWWWWW#
$$$$$$$$&&&&&&&&&###  |           @@@@#######
",
    );
}


#[test]
fn test_scatter_square() {
    render_d2d((0u16..=390).map(scatter_square)).snap(
        r"
    @  #    $@   $$ #  |    # $ #  &  &@      &
 XVWVZ!X%VY!YY!W%YVU   |   X%ZXZXZVXYW!ZWVVU%U 
#%OQONTSSQURTPRPSTOX   |   VOPQNONQRPOTOTSQPPX 
 ZPLIKJLIHKHMHHKJJNU   |   WQLIGMKIHMMJJHMILQ% 
$XQIGEDGFBCCEGDFGJTZ&  |  @VOIGDFEBBGDDDECEKOU 
 ZNGD77798B8AA89DLSW$  |   ZNKE7A877BBA898BJN%$
#XOMFA654364445ACHNX$  |   !TJD76563564657EMS%&
 ZNKE85321111249FLSU&  |  #XSLG753i3211268BLNY 
 VQIB76ii:i::16ACKP!   |   %SIF942i:ii:367FITW#
&XRHB733:...i13BEKPZ#  |   VQHB831:...ii5AFJTY 
 YPMGB52i...:257FHOW   |   YUKCB61i...:269EHN! 
 WOMDB61i...:239DKTV   |   !RHC841:...i249EGO! 
&!TJDA41:i:i:159GIQ%&  |  $YTMEA41:i:::357CIR%#
@ZOJD8623i22323AFKQ$   |  @YPHGA4211221248CLR! 
 WTHE95665645449FLQY#  |   !RHD85463535348CMSZ 
 VSMC8787A99788AGJP!   |   WPKF9A9AB799A9ADIRX 
 VQIEBEBFFEECCCDCLRZ#  |   %SJGDCFCEFDGFFGCMRW 
 UPLKJMLIJHGILMIMMNV   |  $YTJJLHLKKHKIKLJLMRU 
 %PQONSNTTNORRSRRRSY@  |  $VONTSNSPPOTQQQPRNSU&
 UX%U%%YWY!!%!ZXWUUW&  |   UXUZWXU!ZWV%$Y!ZVYW 
&    $& #   #     & $  |  #   &$$& #  & # # @&$
",
    );
}



#[test]
fn test_hilbert() {
    render_d2d((0u16..=390).map(hilbert)).snap(
        r"
....444555666777  |  ..:::iiiOOOOOOQQQQQRWXXXXXZZZZZZ
....444455667777  |  ..:::iiiNNOOPOQQQQRRWWXXXXZYZZ%%
::.:443355668877  |  ...:11iiNNNNPPPPSRRRWWWVYYYY%%%%
::::333355668888  |  ..::111iNNNMPPPPSRRRWWWVYYYY!%%%
::111133BBBB8999  |  44431122MMMMKKKJSSTTUUVV#&&&!!!!
ii111132BBBA8899  |  44431122MMMMKKKJSSTTUUVV#&&&!!!!
iii12222BBAAAA99  |  44333322LLLLKKJJSTTTUUUV##&&$$$$
iiii2222CCAAAA99  |  54333222LLLLLKJJSSTTUUVV##&$$$$$
ONNNMMLLCCDDEEEE  |  5555BBBCCCCCIIIJ        ##      
ONNNMMLLCCDDEEEE  |  5555BBBCCCDCIIJJ        @#      
OONNMMLLCDDDFFEF  |  6666BBAADDDDIIHH        @       
OONMMMLLCCDDFFFF  |  6666BAAADDDDIIHH        @@      
OPPPKKKLIIIIFGGG  |  678888AAEEFFFFHH                
OOPPKKKKIIIIFGGG  |  778898AAEEFFGGHH                
QQPPKKJJIJHHHHGG  |  77789999EEEFGGGH                
QQPPJJJJJJHHHHHG  |  77789999EEFFGGGG                
QQSSSSSS          |  
QQRRSSTS          |  
QRRRTTTT          |  
RRRRTTTT          |  
WWWWUUUU          |  
XWWWUUUU          |  
XXWWVVUV          |  
XXVVVVVV          |  
XXYY#####@@@      |  
XXYY&&#### @      |  
ZZYY&&&&          |  
ZYYY&&&$          |  
ZZ%!!!$$          |  
ZZ%%!!$$          |  
Z%%%!!$$          |  
Z%%%!!$$          |  
",
    );
}

#[test]
fn test_z_order() {
    render_d2d((0u16..=390).map(z_order)).snap(
        r"
...:1122CCDDFGGG  |  ....::ii55556777QQQRSSSSXXYYZZZZ
..::1122CCDDFGGH  |  ....iiii55557777QQRRSSSTXXYYZZ%%
..::2223CCDDGGHH  |  .:::ii1166667788RRRRTTTTYYYZ%%%%
..::2233CDDDGGHH  |  ::::i11166668888RRSSTTTTYYZZ%%%!
:iii3344EEEFHHII  |  112233338899AAABUUUUVVWW!!!!$&&&
:ii13344EEFFHHII  |  112233448999AABBUUUUWWWW!!!!&&&&
ii113444EEFFIIIJ  |  2223444499AABBBBUVVVWWXX$$$$&&##
ii113445EEFFIIJJ  |  2233444599AABBCCVVVVWXXX$$$$####
55668899JJKKMNNO  |  CCCCEEEEJJJJLLLL##@@            
55668999JJKKNNNO  |  CCCDEEEEJJKKLLLL#@@             
556699AAJKKKNNOO  |  DDDDEFFFKKKKMMMM                
556699AAJKKLNNOO  |  DDDDFFFFKKKLMMMM                
6778AABBLLMMOOPP  |  FFGGHHIIMNNNOOPP                
7778AABBLLMMOPPP  |  GGGGHHIINNNNOPPP                
7788ABBCLLMMPPQQ  |  GGHHIIIJNNOOPPQQ                
7788BBBCLLMMPPQQ  |  GHHHIIJJOOOOPPQQ                
QQRRUUUV##        |  
QQRRUUVV#@        |  
QRRSUUVV@@        |  
RRRSUUVV@         |  
SSTTVWWW          |  
SSTTVWWX          |  
SSTTWWXX          |  
STTTWWXX          |  
XXYY!!$$          |  
XXYY!!$$          |  
YYYZ!!$$          |  
YYZZ!!$$          |  
ZZ%%$&&#          |  
ZZ%%&&&#          |  
Z%%%&&##          |  
Z%%!&&##          |  
",
    );
}
