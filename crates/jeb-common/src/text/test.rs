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
    render_d1((0u16..=14).map(zig_zag)).snap(
        r"
             : 
           :   
         :     
       .       
     .         
   .           
 .             
.              
  .            
    .          
      .        
        .      
          :    
            :  
              :
",
    );
}

#[test]
fn test_signedness() {
    render_1d((0u16..=14).map(signedness)).snap(
        r"
.              
 .             
  .            
   .           
    .          
     .         
      .        
       .       
        .      
         :     
          :    
           :   
            :  
             : 
              :
",
    );
}

#[test]
fn test_spiral_square() {
    render_2d((0u16..=390).map(spiral_square)).snap(
        r"
%%%%%%%%%!!!!!!!!!$$
ZSSSSSSSTTTTTTTTTUU$
ZSLLLLLLMMMMMMMMMNU$
ZSLFFFFFFGGGGGGGGNU$
ZRLFAAAAAAABBBBBGNU$
ZRLFA6666666667BHNU$
ZRKFA5222333337BHNU$
ZRKE952iiiiii37BHNU$
ZRKE952i...:i37BHNU&
ZRKE952i...:137CHNV&
YRKE952:...:137CHNV&
YRKE952:::::147CHOV&
YRKE95211111147CHOV&
YQKE95544444447CHOV&
YQKE99888888888CIOV&
YQJEDDDDDDDDDCCCIOV&
YQJJJJJJJJIIIIIIIOV&
YQQQQQPPPPPPPPPOOOV#
YXXXXXXXXXWWWWWWWWW#
         @@@@#######
",
    );
}


#[test]
fn test_scatter_square() {
    render_2d((0u16..=390).map(scatter_square)).snap(
        r"
  # $ #  &  &@      &
 X%ZXZXZVXYW!ZWVVU%U 
 VOPQNONQRPOTOTSQPPX 
 WQLIGMKIHMMJJHMILQ% 
@VOIGDFEBBGDDDECEKOU 
 ZNKE7A877BBA898BJN%$
 !TJD76563564657EMS%&
#XSLG753i3211268BLNY 
 %SIF942i:ii:367FITW#
 VQHB831:...ii5AFJTY 
 YUKCB61i...:269EHN! 
 !RHC841:...i249EGO! 
$YTMEA41:i:::357CIR%#
@YPHGA4211221248CLR! 
 !RHD85463535348CMSZ 
 WPKF9A9AB799A9ADIRX 
 %SJGDCFCEFDGFFGCMRW 
$YTJJLHLKKHKIKLJLMRU 
$VONTSNSPPOTQQQPRNSU&
 UXUZWXU!ZWV%$Y!ZVYW 
#   &$$& #  & # # @&$
",
    );
}



#[test]
fn test_hilbert() {
    render_2d((0u16..=390).map(hilbert)).snap(
        r"
..:::iiiOOOOOOQQQQQRWXXXXXZZZZZZ
..:::iiiNNOOPOQQQQRRWWXXXXZYZZ%%
...:11iiNNNNPPPPSRRRWWWVYYYY%%%%
..::111iNNNMPPPPSRRRWWWVYYYY!%%%
44431122MMMMKKKJSSTTUUVV#&&&!!!!
44431122MMMMKKKJSSTTUUVV#&&&!!!!
44333322LLLLKKJJSTTTUUUV##&&$$$$
54333222LLLLLKJJSSTTUUVV##&$$$$$
5555BBBCCCCCIIIJ        ##      
5555BBBCCCDCIIJJ        @#      
6666BBAADDDDIIHH        @       
6666BAAADDDDIIHH        @@      
678888AAEEFFFFHH                
778898AAEEFFGGHH                
77789999EEEFGGGH                
77789999EEFFGGGG                
",
    );
}
