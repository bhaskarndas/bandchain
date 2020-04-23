package proof

import (
	"testing"
	"time"

	"github.com/stretchr/testify/require"
	"github.com/tendermint/go-amino"
	"github.com/tendermint/tendermint/types"
	"github.com/tendermint/tendermint/version"
)

func TestBlockHeaderMerkleParts(t *testing.T) {
	layout := "2006-01-02T15:04:05.000000000Z"
	str := "2020-04-20T03:30:30.143851745Z"
	blockTime, _ := time.Parse(layout, str)

	// Copy block header Merkle Part here
	header := types.Header{
		Version: version.Consensus{Block: 10, App: 0},
		ChainID: "bandchain",
		Height:  381837,
		Time:    blockTime,
		LastBlockID: types.BlockID{
			Hash: hexToBytes("F633B30D4FBEC862F4A041311E2CB7DFAD63D57930B065A563299449D25BD9CE"),
			PartsHeader: types.PartSetHeader{
				Total: 1,
				Hash:  hexToBytes("7F334B7EE4F8AAC5E70F07FEB9A58A72F120E9AC046167851FC94BC4F2729550"),
			},
		},
		LastCommitHash:     hexToBytes("561D0BB2B6A6E58E20A6BED0F16C8FF5E333BB5A93C69A8E7F3C13542A84DB60"),
		DataHash:           nil,
		ValidatorsHash:     hexToBytes("3AEB137B43144B229F0CA7AC43033E03FCEE25877A3661E88848E436C3D6DD65"),
		NextValidatorsHash: hexToBytes("3AEB137B43144B229F0CA7AC43033E03FCEE25877A3661E88848E436C3D6DD65"),
		ConsensusHash:      hexToBytes("AD82B220C509602720D74FD75BCE7CFE9B148039958F236D8894E00EB1599E04"),
		AppHash:            hexToBytes("1CCD765C80D0DC1705BB7B6BE616DAD3CF2E6439BB9A9B776D5BD183F89CA141"),
		LastResultsHash:    nil,
		EvidenceHash:       nil,
		ProposerAddress:    hexToBytes("F23391B5DBF982E37FB7DADEA64AAE21CAE4C172"),
	}
	blockMerkleParts := GetBlockHeaderMerkleParts(amino.NewCodec(), &header)
	expectBlockHash := hexToBytes("A35617A81409CE46F1F820450B8AD4B217D99AE38AAA719B33C4FC52DCA99B22")
	appHash := hexToBytes("1CCD765C80D0DC1705BB7B6BE616DAD3CF2E6439BB9A9B776D5BD183F89CA141")
	blockHeight := 381837

	// Verify code
	blockHash := branchHash(
		branchHash(
			branchHash(
				blockMerkleParts.VersionAndChainIdHash,
				branchHash(
					leafHash(cdcEncode(amino.NewCodec(), blockHeight)),
					blockMerkleParts.TimeHash,
				),
			),
			blockMerkleParts.LastBlockIDAndOther,
		),
		branchHash(
			branchHash(
				blockMerkleParts.NextValidatorHashAndConsensusHash,
				branchHash(
					leafHash(cdcEncode(amino.NewCodec(), appHash)),
					blockMerkleParts.LastResultsHash,
				),
			),
			blockMerkleParts.EvidenceAndProposerHash,
		),
	)
	require.Equal(t, expectBlockHash, blockHash)
}
